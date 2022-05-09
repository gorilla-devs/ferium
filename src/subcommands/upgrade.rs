use crate::{mutex_ext::MutexExt, CROSS, TICK, YELLOW_TICK};
use anyhow::{bail, Error, Result};
use colored::Colorize;
use ferinth::Ferinth;
use fs_extra::file::{move_file, CopyOptions};
use furse::Furse;
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
use tokio::{fs::copy, spawn};

pub async fn upgrade(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    github: Arc<Octocrab>,
    profile: &config::structs::Profile,
) -> Result<()> {
    let profile = Arc::new(profile.clone());
    let to_download = Arc::new(Mutex::new(Vec::new()));
    let backwards_compat_msg = Arc::new(AtomicBool::new(false));
    let error = Arc::new(AtomicBool::new(false));
    let mut tasks = Vec::new();

    println!("{}\n", "Determining the Latest Compatible Versions".bold());
    for mod_ in &profile.mods {
        let backwards_compat_msg = backwards_compat_msg.clone();
        let to_download = to_download.clone();
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
            match result {
                Ok((downloadable, backwards_compat)) => {
                    println!(
                        "{} {:45} {}",
                        if backwards_compat {
                            backwards_compat_msg.store(true, Ordering::Relaxed);
                            YELLOW_TICK.clone()
                        } else {
                            TICK.clone()
                        },
                        mod_.name,
                        format!("({})", downloadable.filename).dimmed()
                    );
                    {
                        let mut to_download = to_download.force_lock();
                        to_download.push(downloadable);
                    }
                },
                Err(err) => {
                    if let upgrade::Error::ModrinthError(ferinth::Error::RateLimitExceeded(_)) = err
                    {
                        // Immediately fail if there is a rate limit
                        bail!(err);
                    }
                    eprintln!("{}", format!("{} {:45} {}", CROSS, mod_.name, err).red());
                    error.store(true, Ordering::Relaxed);
                },
            }
            Ok(())
        }));
    }
    for handle in tasks {
        handle.await??;
    }
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
            } else {
                let _ = move_file(
                    file.path(),
                    profile.output_dir.join(".old").join(filename),
                    &CopyOptions::new(),
                );
            }
        }
    }

    if to_download.is_empty() && to_install.is_empty() {
        println!("\n{}", "All up to date!".bold());
    } else {
        println!("\n{}\n", "Downloading Mod Files".bold());
        let mut tasks = Vec::new();
        for downloadable in to_download {
            let profile = profile.clone();
            let downloadable = downloadable.clone();
            tasks.push(spawn(async move {
                let contents = reqwest::get(&downloadable.download_url)
                    .await?
                    .bytes()
                    .await?;
                let size = Size::Bytes(contents.len());
                check::write_mod_file(&profile.output_dir, contents, &downloadable.filename)
                    .await?;
                println!(
                    "{} Downloaded {:7} {}",
                    &*TICK,
                    size.to_string(Base::Base10, Style::Smart),
                    downloadable.filename.dimmed(),
                );
                Ok::<(), Error>(())
            }));
        }
        for handle in tasks {
            handle.await??;
        }
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
