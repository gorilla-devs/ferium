// Allow `expect()`s for mutex poisons
#![allow(clippy::expect_used)]

use crate::{
    download::{clean, download},
    CROSS, STYLE_NO, TICK, YELLOW_TICK,
};
use anyhow::{anyhow, bail, Result};
use colored::Colorize;
use ferinth::Ferinth;
use furse::Furse;
use indicatif::ProgressBar;
use libium::{
    config::structs::{ModLoader, Profile},
    upgrade::{
        mod_downloadable::{self, get_latest_compatible_downloadable},
        Downloadable,
    },
};
use octocrab::Octocrab;
use std::{
    fs::read_dir,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{sync::Semaphore, task::JoinSet};

/// Get the latest compatible downloadable for the mods in `profile`
///
/// If an error occurs with a resolving task, instead of failing immediately,
/// resolution will continue and the error return flag is set to true.
pub async fn get_platform_downloadables(
    modrinth: Ferinth,
    curseforge: Furse,
    github: Octocrab,
    profile: &Profile,
) -> Result<(Vec<Downloadable>, bool)> {
    let to_download = Arc::new(Mutex::new(Vec::new()));
    let progress_bar = Arc::new(Mutex::new(
        ProgressBar::new(profile.mods.len() as u64).with_style(STYLE_NO.clone()),
    ));
    let mut tasks = JoinSet::new();
    let profile = Arc::new(profile.clone());
    let curseforge = Arc::new(curseforge);
    let modrinth = Arc::new(modrinth);
    let github = Arc::new(github);

    println!("{}\n", "Determining the Latest Compatible Versions".bold());
    let semaphore = Arc::new(Semaphore::new(75));
    progress_bar
        .lock()
        .expect("Mutex poisoned")
        .enable_steady_tick(Duration::from_millis(100));
    let pad_len = profile
        .mods
        .iter()
        .map(|m| m.name.len())
        .max()
        .unwrap_or(20)
        .clamp(20, 50);
    for mod_ in &profile.mods {
        let permit = semaphore.clone().acquire_owned().await?;
        let to_download = to_download.clone();
        let progress_bar = progress_bar.clone();
        let curseforge = curseforge.clone();
        let modrinth = modrinth.clone();
        let profile = profile.clone();
        let github = github.clone();
        let mod_ = mod_.clone();
        tasks.spawn(async move {
            let _permit = permit;
            let result = get_latest_compatible_downloadable(
                &modrinth,
                &curseforge,
                &github,
                &mod_,
                &profile.game_version,
                profile.mod_loader,
            )
            .await;
            let progress_bar = progress_bar.lock().expect("Mutex poisoned");
            progress_bar.inc(1);
            match result {
                Ok((downloadable, qf_flag)) => {
                    progress_bar.println(format!(
                        "{} {:pad_len$}  {}",
                        if qf_flag {
                            YELLOW_TICK.clone()
                        } else {
                            TICK.clone()
                        },
                        mod_.name,
                        downloadable.filename().dimmed()
                    ));
                    {
                        to_download
                            .lock()
                            .expect("Mutex poisoned")
                            .push(downloadable);
                        Ok((false, qf_flag))
                    }
                }
                Err(err) => {
                    if let mod_downloadable::Error::ModrinthError(
                        ferinth::Error::RateLimitExceeded(_),
                    ) = err
                    {
                        // Immediately fail if the rate limit has been exceeded
                        progress_bar.finish_and_clear();
                        bail!(err);
                    }
                    progress_bar.println(format!(
                        "{}",
                        format!("{CROSS} {:pad_len$}  {err}", mod_.name).red()
                    ));
                    Ok((true, false))
                }
            }
        });
    }

    let mut error = false;
    let mut qf_flag = false;
    while let Some(res) = tasks.join_next().await {
        let res = res??;
        error |= res.0;
        qf_flag |= res.1;
    }
    Arc::try_unwrap(progress_bar)
        .map_err(|_| anyhow!("Failed to run threads to completion"))?
        .into_inner()?
        .finish_and_clear();
    if qf_flag {
        println!(
            "{}",
            "Fabric mod using Quilt backwards compatibility".yellow()
        );
    }
    Ok((
        Arc::try_unwrap(to_download)
            .map_err(|_| anyhow!("Failed to run threads to completion"))?
            .into_inner()?,
        error,
    ))
}

pub async fn upgrade(
    modrinth: Ferinth,
    curseforge: Furse,
    github: Octocrab,
    profile: &Profile,
) -> Result<()> {
    let (mut to_download, error) =
        get_platform_downloadables(modrinth, curseforge, github, profile).await?;
    let mut to_install = Vec::new();
    if profile.output_dir.join("user").exists() && profile.mod_loader != ModLoader::Quilt {
        for file in read_dir(profile.output_dir.join("user"))? {
            let file = file?;
            let path = file.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "jar") {
                to_install.push((file.file_name(), path));
            }
        }
    }

    clean(&profile.output_dir, &mut to_download, &mut to_install).await?;
    to_download
        .iter_mut()
        // Download directly to the output directory
        .map(|thing| thing.output = thing.filename().into())
        .for_each(drop); // Doesn't drop any data, just runs the iterator
    if to_download.is_empty() && to_install.is_empty() {
        println!("\n{}", "All up to date!".bold());
    } else {
        println!("\n{}\n", "Downloading Mod Files".bold());
        download(profile.output_dir.clone(), to_download, to_install).await?;
    }

    if error {
        bail!("\nCould not get the latest compatible version of some mods")
    }

    Ok(())
}
