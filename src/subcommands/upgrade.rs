use crate::{
    download::{clean, download},
    CROSS, STYLE_NO, TICK, YELLOW_TICK,
};
use anyhow::{anyhow, bail, Result};
use colored::Colorize;
use ferinth::{
    structures::version::{Version, VersionFile},
    Ferinth,
};
use furse::{structures::file_structs::File, Furse};
use indicatif::ProgressBar;
use libium::{
    config::structs::{ModIdentifier, ModLoader, Profile},
    mutex_ext::MutexExt,
    upgrade::{mod_downloadable, DistributionDeniedError, Downloadable},
};
use octocrab::{models::repos::Asset, Octocrab};
use std::{
    fs::read_dir,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::{sync::Semaphore, task::JoinSet};

pub enum PlatformDownloadable {
    Modrinth(VersionFile, Version),
    CurseForge(File),
    GitHub(Asset),
}

impl PlatformDownloadable {
    pub fn filename(&self) -> &str {
        match self {
            Self::Modrinth(version_file, _) => &version_file.filename,
            Self::CurseForge(file) => &file.file_name,
            Self::GitHub(asset) => &asset.name,
        }
    }
}

impl TryFrom<PlatformDownloadable> for Downloadable {
    type Error = DistributionDeniedError;
    fn try_from(platform_downloadable: PlatformDownloadable) -> Result<Self, Self::Error> {
        match platform_downloadable {
            PlatformDownloadable::Modrinth(version_file, _) => Ok(version_file.into()),
            PlatformDownloadable::CurseForge(file) => file.try_into(),
            PlatformDownloadable::GitHub(asset) => Ok(asset.into()),
        }
    }
}

/// Get the latest compatible downloadable for the mods in `profile`
///
/// If an error occures with a resolving task, instead of failing immediately, the error return flag is set to true
pub async fn get_platform_downloadables(
    modrinth: Ferinth,
    curseforge: Furse,
    github: Octocrab,
    profile: &Profile,
) -> Result<(Vec<PlatformDownloadable>, bool)> {
    let profile = Arc::new(profile.clone());
    let to_download = Arc::new(Mutex::new(Vec::new()));
    let progress_bar = Arc::new(Mutex::new(
        ProgressBar::new(profile.mods.len() as u64).with_style(STYLE_NO.clone()),
    ));
    let backwards_compat_msg = Arc::new(AtomicBool::new(false));
    let error = Arc::new(AtomicBool::new(false));
    let mut tasks = JoinSet::new();
    let curseforge = Arc::new(curseforge);
    let modrinth = Arc::new(modrinth);
    let github = Arc::new(github);

    println!("{}\n", "Determining the Latest Compatible Versions".bold());
    let semaphore = Arc::new(Semaphore::new(75));
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
        tasks.spawn(async move {
            let game_version_to_check = if mod_.check_game_version == Some(false) {
                None
            } else {
                Some(profile.game_version.as_ref())
            };
            let mod_loader_to_check = if mod_.check_mod_loader == Some(false) {
                None
            } else {
                Some(&profile.mod_loader)
            };
            let _permit = permit;
            let result = match &mod_.identifier {
                ModIdentifier::CurseForgeProject(project_id) => {
                    mod_downloadable::get_latest_compatible_file(
                        curseforge.get_mod_files(*project_id).await?,
                        game_version_to_check,
                        mod_loader_to_check,
                    )
                    .map_or_else(
                        || Err(mod_downloadable::Error::NoCompatibleFile),
                        |ok| Ok((PlatformDownloadable::CurseForge(ok.0), ok.1)),
                    )
                }
                ModIdentifier::ModrinthProject(project_id) => {
                    mod_downloadable::get_latest_compatible_version(
                        &modrinth.list_versions(project_id).await?,
                        game_version_to_check,
                        mod_loader_to_check,
                    )
                    .map_or_else(
                        || Err(mod_downloadable::Error::NoCompatibleFile),
                        |ok| Ok((PlatformDownloadable::Modrinth(ok.0, ok.1), ok.2)),
                    )
                }
                ModIdentifier::GitHubRepository(full_name) => {
                    mod_downloadable::get_latest_compatible_asset(
                        &github
                            .repos(&full_name.0, &full_name.1)
                            .releases()
                            .list()
                            .send()
                            .await?
                            .items,
                        game_version_to_check,
                        mod_loader_to_check,
                    )
                    .map_or_else(
                        || Err(mod_downloadable::Error::NoCompatibleFile),
                        |ok| Ok((PlatformDownloadable::GitHub(ok.0), ok.1)),
                    )
                }
            };
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
                }
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
                        format!("{CROSS} {:43} {err}", mod_.name).red()
                    ));
                    error.store(true, Ordering::Relaxed);
                }
            }
            progress_bar.inc(1);
            Ok(())
        });
    }
    while let Some(res) = tasks.join_next().await {
        res??;
    }
    Arc::try_unwrap(progress_bar)
        .map_err(|_| anyhow!("Failed to run threads to completion"))?
        .into_inner()?
        .finish_and_clear();
    if backwards_compat_msg.load(Ordering::Relaxed) {
        println!(
            "{}",
            "Fabric mod using Quilt backwards compatibility".yellow()
        );
    }
    Ok((
        Arc::try_unwrap(to_download)
            .map_err(|_| anyhow!("Failed to run threads to completion"))?
            .into_inner()?,
        error.load(Ordering::Relaxed),
    ))
}

pub async fn upgrade(
    modrinth: Ferinth,
    curseforge: Furse,
    github: Octocrab,
    profile: &Profile,
) -> Result<()> {
    let (to_download, error) =
        get_platform_downloadables(modrinth, curseforge, github, profile).await?;
    let mut to_download = to_download
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>, _>>()?;
    let mut to_install = Vec::new();
    if profile.output_dir.join("user").exists() && profile.mod_loader != ModLoader::Quilt {
        for file in read_dir(profile.output_dir.join("user"))? {
            let file = file?;
            let path = file.path();
            if path.is_file() {
                // TODO: Use `path.extension().is_some_and(|ext| ext == "jar")` instead, see [#93050](https://github.com/rust-lang/rust/issues/93050)
                if let Some(ext) = path.extension() {
                    if ext == "jar" {
                        to_install.push((file.file_name(), path));
                    }
                }
            }
        }
    }

    clean(&profile.output_dir, &mut to_download, &mut to_install).await?;
    to_download
        .iter_mut()
        // Download directly to the output directory
        .map(|thing| thing.output = thing.filename().into())
        .for_each(drop); // Doesn't drop any data, just runs the interator
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
