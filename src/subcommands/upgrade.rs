#![expect(clippy::expect_used, reason = "For mutex poisons")]

use crate::{
    download::{clean, download},
    CROSS, STYLE_NO, TICK,
};
use anyhow::{anyhow, bail, Result};
use colored::Colorize as _;
use futures::{stream::FuturesUnordered, StreamExt as _, TryFutureExt as _};
use indicatif::ProgressBar;
use libium::{
    config::{
        filters::ProfileParameters as _,
        structs::{ModLoader, Profile},
    },
    iter_ext::IterExt as _,
    upgrade::{check, mod_downloadable, DownloadFile},
};
use std::{
    fs::read_dir,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::Semaphore;

/// Get the latest compatible downloadable for the mods in `profile`
///
/// If an error occurs with a resolving task, instead of failing immediately,
/// resolution will continue and the error return flag is set to true.
pub async fn get_platform_downloadables(profile: &Profile) -> Result<(Vec<DownloadFile>, bool)> {
    let to_download = Arc::new(Mutex::new(Vec::new()));
    let progress_bar = Arc::new(Mutex::new(
        ProgressBar::new(profile.mods.len() as u64).with_style(STYLE_NO.clone()),
    ));
    let mut tasks = FuturesUnordered::new();

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
    for mod_ in profile.mods.clone() {
        let permit = Arc::clone(&semaphore).acquire_owned().await?;
        let to_download = Arc::clone(&to_download);
        let progress_bar = Arc::clone(&progress_bar);
        let filters = if mod_.override_filters {
            mod_.filters
        } else {
            profile
                .filters
                .clone()
                .into_iter()
                .chain(mod_.filters)
                .collect_vec()
        };

        tasks.push(async move {
            let _permit = permit;
            let result = mod_
                .identifier
                .fetch_download_files()
                .map_err(anyhow::Error::from)
                .and_then(|f| check::select_latest(f, filters).map_err(anyhow::Error::from))
                .await;
            let progress_bar = progress_bar.lock().expect("Mutex poisoned");
            progress_bar.inc(1);
            match result {
                Ok(download_file) => {
                    progress_bar.println(format!(
                        "{} {:pad_len$}  {}",
                        TICK.clone(),
                        mod_.name,
                        download_file.filename().dimmed()
                    ));
                    to_download
                        .lock()
                        .expect("Mutex poisoned")
                        .push(download_file);
                    Ok(true)
                }
                Err(err) => {
                    if let Some(mod_downloadable::Error::ModrinthError(
                        ferinth::Error::RateLimitExceeded(_),
                    )) = err.downcast_ref()
                    {
                        // Immediately fail if the rate limit has been exceeded
                        progress_bar.finish_and_clear();
                        bail!(err);
                    }
                    progress_bar.println(format!(
                        "{}",
                        format!("{CROSS} {:pad_len$}  {err}", mod_.name).red()
                    ));
                    Ok(false)
                }
            }
        });
    }

    let mut error = false;
    while let Some(res) = tasks.next().await {
        let res = res?;
        error |= !res;
    }
    Arc::try_unwrap(progress_bar)
        .map_err(|_| anyhow!("Failed to run threads to completion"))?
        .into_inner()?
        .finish_and_clear();
    Ok((
        Arc::try_unwrap(to_download)
            .map_err(|_| anyhow!("Failed to run threads to completion"))?
            .into_inner()?,
        error,
    ))
}

pub async fn upgrade(profile: &Profile) -> Result<()> {
    let (mut to_download, error) = get_platform_downloadables(profile).await?;
    let mut to_install = Vec::new();
    if profile.output_dir.join("user").exists()
        && profile.filters.mod_loader() != Some(&ModLoader::Quilt)
    {
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
        Err(anyhow!(
            "\nCould not get the latest compatible version of some mods"
        ))
    } else {
        Ok(())
    }
}
