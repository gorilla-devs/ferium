use crate::{
    download::{clean, download},
    style_no, CROSS, TICK, YELLOW_TICK,
};
use anyhow::{anyhow, bail, Result};
use colored::Colorize;
use ferinth::Ferinth;
use furse::Furse;
use indicatif::ProgressBar;
use libium::{config::structs::Profile, mutex_ext::MutexExt, upgrade::mod_downloadable};
use octocrab::Octocrab;
use std::{
    fs::read_dir,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::{spawn, sync::Semaphore};

pub async fn upgrade(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    github: Arc<Octocrab>,
    profile: &Profile,
) -> Result<()> {
    let profile = Arc::new(profile.clone());
    let to_download = Arc::new(Mutex::new(Vec::new()));
    let progress_bar = Arc::new(Mutex::new(
        ProgressBar::new(profile.mods.len() as u64).with_style(style_no()),
    ));
    let backwards_compat_msg = Arc::new(AtomicBool::new(false));
    let error = Arc::new(AtomicBool::new(false));
    let mut tasks = Vec::new();

    println!("{}\n", "Determining the Latest Compatible Versions".bold());
    let semaphore = Arc::new(Semaphore::new(40));
    progress_bar
        .force_lock()
        .enable_steady_tick(Duration::from_millis(100));
    for mod_ in &profile.mods {
        let permit = semaphore.clone().acquire_owned().await?;
        let backwards_compat_msg = backwards_compat_msg.clone();
        let to_download = to_download.clone();
        let progress_bar = progress_bar.clone();
        let curseforge = curseforge.clone();
        let modrinth = modrinth.clone();
        let profile = profile.clone();
        let error = error.clone();
        let github = github.clone();
        let mod_ = mod_.clone();
        tasks.push(spawn(async move {
            let _permit = permit;
            let result = mod_downloadable::get_latest_compatible_downloadable(
                modrinth.clone(),
                curseforge.clone(),
                github.clone(),
                &mod_,
                &profile.game_version,
                &profile.mod_loader,
            )
            .await;
            let progress_bar = progress_bar.force_lock();
            match result {
                Ok((downloadable, backwards_compat)) => {
                    progress_bar.println(format!(
                        "{} {:43} {}",
                        if backwards_compat {
                            backwards_compat_msg.store(true, Ordering::Relaxed);
                            YELLOW_TICK.clone()
                        } else {
                            TICK.clone()
                        },
                        mod_.name,
                        downloadable.filename().dimmed()
                    ));
                    {
                        let mut to_download = to_download.force_lock();
                        to_download.push(downloadable);
                    }
                },
                Err(err) => {
                    if let mod_downloadable::Error::ModrinthError(
                        ferinth::Error::RateLimitExceeded(_),
                    ) = err
                    {
                        // Immediately fail if there is a rate limit
                        progress_bar.finish_and_clear();
                        bail!(err);
                    }
                    progress_bar.println(format!(
                        "{}",
                        format!("{} {:43} {}", CROSS, mod_.name, err).red()
                    ));
                    error.store(true, Ordering::Relaxed);
                },
            }
            progress_bar.set_position(progress_bar.position() + 1);
            Ok(())
        }));
    }
    for handle in tasks {
        handle.await??;
    }
    Arc::try_unwrap(progress_bar)
        .map_err(|_| anyhow!("Failed to run threads to completion"))?
        .into_inner()?
        .finish_and_clear();
    let mut to_download = Arc::try_unwrap(to_download)
        .map_err(|_| anyhow!("Failed to run threads to completion"))?
        .into_inner()?;
    if backwards_compat_msg.load(Ordering::Relaxed) {
        println!(
            "{}",
            "Fabric mod using Quilt backwards compatibility".yellow()
        );
    }

    let mut to_install = Vec::new();
    if profile.output_dir.join("user").exists() {
        for file in read_dir(&profile.output_dir.join("user"))? {
            let file = file?;
            let path = file.path();
            if path.is_file() {
                to_install.push((file.file_name(), path));
            }
        }
    }

    clean(&profile.output_dir, &mut to_download, &mut to_install).await?;
    to_download
        .iter_mut()
        .map(|thing| thing.output = thing.filename().into())
        .for_each(drop);
    if to_download.is_empty() && to_install.is_empty() {
        println!("\n{}", "All up to date!".bold());
    } else {
        println!("\n{}\n", "Downloading Mod Files".bold());
        download(
            Arc::new(profile.output_dir.clone()),
            to_download,
            to_install,
        )
        .await?;
    }

    if error.load(Ordering::Relaxed) {
        bail!("\nCould not get the latest compatible version of some mods")
    }

    Ok(())
}
