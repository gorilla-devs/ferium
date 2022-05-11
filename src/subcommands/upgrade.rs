use crate::{mutex_ext::MutexExt, CROSS, STYLE, TICK, YELLOW_TICK};
use anyhow::{bail, Error, Result};
use colored::Colorize;
use ferinth::Ferinth;
use fs_extra::file::{move_file, CopyOptions};
use furse::Furse;
use indicatif::ProgressBar;
use itertools::Itertools;
use libium::{check, config, upgrade};
use octocrab::Octocrab;
use size::{Base, Size, Style};
use std::{
    fs::read_dir,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tokio::{
    fs::{copy, remove_file},
    spawn,
};

pub async fn upgrade(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    github: Arc<Octocrab>,
    profile: &config::structs::Profile,
) -> Result<()> {
    let profile = Arc::new(profile.clone());
    let to_download = Arc::new(Mutex::new(Vec::new()));
    let progress_bar = Arc::new(Mutex::new(
        ProgressBar::new(profile.mods.len() as u64).with_style(STYLE.clone()),
    ));
    let backwards_compat_msg = Arc::new(AtomicBool::new(false));
    let error = Arc::new(AtomicBool::new(false));
    let mut tasks = Vec::new();

    println!("{}\n", "Determining the Latest Compatible Versions".bold());
    {
        progress_bar.force_lock().enable_steady_tick(100);
    }
    for mod_ in &profile.mods {
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
            let result = upgrade::get_latest_compatible_downloadable(
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
                        "{} {:45} {}",
                        if backwards_compat {
                            backwards_compat_msg.store(true, Ordering::Relaxed);
                            YELLOW_TICK.clone()
                        } else {
                            TICK.clone()
                        },
                        mod_.name,
                        format!("({})", downloadable.filename).dimmed()
                    ));
                    {
                        let mut to_download = to_download.force_lock();
                        to_download.push(downloadable);
                    }
                },
                Err(err) => {
                    if let upgrade::Error::ModrinthError(ferinth::Error::RateLimitExceeded(_)) = err
                    {
                        // Immediately fail if there is a rate limit
                        progress_bar.finish_and_clear();
                        bail!(err);
                    }
                    progress_bar.println(format!(
                        "{}",
                        format!("{} {:45} {}", CROSS, mod_.name, err).red()
                    ));
                    error.store(true, Ordering::Relaxed);
                },
            }
            progress_bar.inc(1);
            Ok(())
        }));
    }
    for handle in tasks {
        handle.await??;
    }
    Arc::try_unwrap(progress_bar)
        .expect("Failed to run threads to completion")
        .into_inner()?
        .finish_and_clear();
    let mut to_download = Arc::try_unwrap(to_download)
        .expect("Failed to run threads to completion")
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

    for file in read_dir(&profile.output_dir)? {
        let file = file?;
        if file.file_type()?.is_file() {
            let filename = file.file_name();
            let filename = filename.to_str().unwrap();
            if let Some((index, _)) = to_download
                .iter()
                .find_position(|thing| filename == thing.filename)
            {
                to_download.swap_remove(index);
            } else if let Some((index, _)) =
                to_install.iter().find_position(|thing| filename == thing.0)
            {
                to_install.swap_remove(index);
            } else if move_file(
                file.path(),
                profile.output_dir.join(".old").join(filename),
                &CopyOptions::new(),
            )
            .is_err()
            {
                remove_file(file.path()).await?;
            }
        }
    }

    if to_download.is_empty() && to_install.is_empty() {
        println!("\n{}", "All up to date!".bold());
    } else {
        println!("\n{}\n", "Downloading Mod Files".bold());
        let progress_bar = Arc::new(Mutex::new(
            ProgressBar::new(to_download.len() as u64).with_style(STYLE.clone()),
        ));
        {
            progress_bar.force_lock().enable_steady_tick(100);
        }
        let mut tasks = Vec::new();
        for downloadable in to_download {
            let progress_bar = progress_bar.clone();
            let downloadable = downloadable.clone();
            let profile = profile.clone();
            tasks.push(spawn(async move {
                let contents = reqwest::get(&downloadable.download_url)
                    .await?
                    .bytes()
                    .await?;
                let size = Size::Bytes(contents.len());
                check::write_mod_file(&profile.output_dir, contents, &downloadable.filename)
                    .await?;
                let progress_bar = progress_bar.force_lock();
                progress_bar.println(format!(
                    "{} Downloaded {:7} {}",
                    &*TICK,
                    size.to_string(Base::Base10, Style::Smart),
                    downloadable.filename.dimmed(),
                ));
                progress_bar.inc(1);
                Ok::<(), Error>(())
            }));
        }
        for handle in tasks {
            handle.await??;
        }
        Arc::try_unwrap(progress_bar)
            .expect("Failed to run threads to completion")
            .into_inner()?
            .finish_and_clear();
        for installable in to_install {
            eprint!(
                "Installing  {}... ",
                installable.0.to_string_lossy().dimmed()
            );
            copy(installable.1, profile.output_dir.join(installable.0)).await?;
            println!("{}", &*TICK);
        }
    }

    if error.load(Ordering::Relaxed) {
        bail!("\nCould not get the latest compatible version of some mods")
    }

    Ok(())
}
