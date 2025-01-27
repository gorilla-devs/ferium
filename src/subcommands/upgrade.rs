use crate::{
    download::{clean, download},
    CROSS, DEFAULT_PARALLEL_NETWORK, SEMAPHORE, STYLE_NO, TICK,
};
use anyhow::{anyhow, bail, Result};
use colored::Colorize as _;
use indicatif::ProgressBar;
use libium::{
    config::{
        filters::ProfileParameters as _,
        structs::{Mod, ModIdentifier, ModLoader, Profile, ProfileItem},
    },
    upgrade::{mod_downloadable, DownloadData},
};
use parking_lot::Mutex;
use std::{
    fs::read_dir,
    mem::take,
    sync::{mpsc, Arc},
    time::Duration,
};
use tokio::{sync::Semaphore, task::JoinSet};

/// Get the latest compatible downloadable for the mods in `profile`
///
/// If an error occurs with a resolving task, instead of failing immediately,
/// resolution will continue and the error return flag is set to true.
pub async fn get_platform_downloadables(profile: &Profile) -> Result<(Vec<DownloadData>, bool)> {
    let to_download = Arc::new(Mutex::new(Vec::new()));
    let progress_bar = Arc::new(Mutex::new(ProgressBar::new(0).with_style(STYLE_NO.clone())));
    let mut tasks = JoinSet::new();
    let mut done_mods = Vec::new();
    let (mod_sender, mod_rcvr) = mpsc::channel();

    // Wrap it again in an Arc so that I can count the references to it,
    // because I cannot drop the main thread's sender due to the recursion
    let mod_sender = Arc::new(mod_sender);

    println!("{}\n", "Determining the Latest Compatible Versions".bold());
    progress_bar
        .lock()
        .enable_steady_tick(Duration::from_millis(100));
    let pad_len = profile
        .mods
        .iter()
        .map(|m| m.name.len())
        .max()
        .unwrap_or(20)
        .clamp(20, 50);

    for mod_ in profile.mods.clone() {
        mod_sender.send(mod_)?;
    }

    let mut initial = true;

    // A race condition exists where if the last task drops its sender before this thread receives the message,
    // that particular message will get ignored. I used the ostrich algorithm to solve this.

    // `initial` accounts for the edge case where at first,
    // no tasks have been spawned yet but there are messages in the channel
    while Arc::strong_count(&mod_sender) > 1 || initial {
        if let Ok(mod_) = mod_rcvr.try_recv() {
            initial = false;

            if done_mods.contains(&mod_.identifier) {
                continue;
            }

            done_mods.push(mod_.identifier.clone());
            progress_bar.lock().inc_length(1);

            let filters = profile.filters.clone();
            let dep_sender = Arc::clone(&mod_sender);
            let to_download = Arc::clone(&to_download);
            let progress_bar = Arc::clone(&progress_bar);

            tasks.spawn(async move {
                let permit = SEMAPHORE
                    .get_or_init(|| Semaphore::new(DEFAULT_PARALLEL_NETWORK))
                    .acquire()
                    .await?;

                let result = mod_.fetch_download_file(filters).await;

                drop(permit);

                progress_bar.lock().inc(1);
                match result {
                    Ok(mut download_file) => {
                        progress_bar.lock().println(format!(
                            "{} {:pad_len$}  {}",
                            TICK.clone(),
                            mod_.name,
                            download_file.filename().dimmed()
                        ));
                        for dep in take(&mut download_file.dependencies) {
                            dep_sender.send(Mod::new(
                                format!(
                                    "Dependency: {}",
                                    match &dep {
                                        ModIdentifier::CurseForgeProject(id) => id.to_string(),
                                        ModIdentifier::ModrinthProject(id)
                                        | ModIdentifier::PinnedModrinthProject(id, _) =>
                                            id.to_owned(),
                                        _ => unreachable!(),
                                    }
                                ),
                                match dep {
                                    ModIdentifier::PinnedModrinthProject(id, _) => {
                                        ModIdentifier::ModrinthProject(id)
                                    }
                                    _ => dep,
                                },
                                vec![],
                                false,
                            ))?;
                        }
                        to_download.lock().push(download_file);
                        Ok(true)
                    }
                    Err(err) => {
                        if let mod_downloadable::Error::ModrinthError(
                            ferinth::Error::RateLimitExceeded(_),
                        ) = err
                        {
                            // Immediately fail if the rate limit has been exceeded
                            progress_bar.lock().finish_and_clear();
                            bail!(err);
                        }
                        progress_bar.lock().println(format!(
                            "{}",
                            format!("{CROSS} {:pad_len$}  {err}", mod_.name).red()
                        ));
                        Ok(false)
                    }
                }
            });
        }
    }

    let error = tasks
        .join_all()
        .await
        .iter()
        .any(|r| matches!(r, Ok(false)));

    Arc::try_unwrap(progress_bar)
        .map_err(|_| anyhow!("Failed to run threads to completion"))?
        .into_inner()
        .finish_and_clear();
    Ok((
        Arc::try_unwrap(to_download)
            .map_err(|_| anyhow!("Failed to run threads to completion"))?
            .into_inner(),
        error,
    ))
}

pub async fn upgrade(profile_item: &ProfileItem, profile: &Profile) -> Result<()> {
    let (mut to_download, error) = get_platform_downloadables(profile).await?;
    let mut to_install = Vec::new();
    if profile_item.output_dir.join("user").exists()
        && profile.filters.mod_loader() != Some(&ModLoader::Quilt)
    {
        for file in read_dir(profile_item.output_dir.join("user"))? {
            let file = file?;
            let path = file.path();
            if path.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
            {
                to_install.push((file.file_name(), path));
            }
        }
    }

    clean(&profile_item.output_dir, &mut to_download, &mut to_install).await?;
    to_download
        .iter_mut()
        // Download directly to the output directory
        .map(|thing| thing.output = thing.filename().into())
        .for_each(drop); // Doesn't drop any data, just runs the iterator
    if to_download.is_empty() && to_install.is_empty() {
        println!("\n{}", "All up to date!".bold());
    } else {
        println!("\n{}\n", "Downloading Mod Files".bold());
        download(profile_item.output_dir.clone(), to_download, to_install).await?;
    }

    if error {
        Err(anyhow!(
            "\nCould not get the latest compatible version of some mods"
        ))
    } else {
        Ok(())
    }
}
