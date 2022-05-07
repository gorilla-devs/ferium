use crate::{CROSS, TICK, YELLOW_TICK};
use anyhow::{bail, Result};
use colored::Colorize;
use ferinth::Ferinth;
use fs_extra::file::{move_file, CopyOptions};
use furse::Furse;
use itertools::Itertools;
use libium::config;
use libium::upgrade;
use octocrab::Octocrab;
use std::fs::read_dir;
use tokio::fs::copy;

struct Downloadable {
    filename: String,
    download_url: String,
}
impl From<furse::structures::file_structs::File> for Downloadable {
    fn from(file: furse::structures::file_structs::File) -> Self {
        Self {
            filename: file.file_name,
            download_url: file.download_url,
        }
    }
}
impl From<ferinth::structures::version_structs::Version> for Downloadable {
    #[allow(clippy::redundant_else)] // The `else` makes it more readable
    fn from(version: ferinth::structures::version_structs::Version) -> Self {
        let mut files = Vec::new();
        for file in version.files {
            if file.primary {
                return Self {
                    filename: file.filename,
                    download_url: file.url,
                };
            } else {
                files.push(file);
            }
        }
        let file = files.remove(0);
        Self {
            filename: file.filename,
            download_url: file.url,
        }
    }
}
impl From<octocrab::models::repos::Asset> for Downloadable {
    fn from(asset: octocrab::models::repos::Asset) -> Self {
        Self {
            filename: asset.name,
            download_url: asset.browser_download_url.into(),
        }
    }
}

pub async fn upgrade(
    modrinth: &Ferinth,
    curseforge: &Furse,
    github: &Octocrab,
    profile: &config::structs::Profile,
) -> Result<()> {
    let mut to_download = Vec::new();
    let mut backwards_compat_msg = false;
    let mut error = false;

    println!("{}\n", "Determining the Latest Compatible Versions".bold());
    for mod_ in &profile.mods {
        use libium::config::structs::ModIdentifier;
        let (result, backwards_compat): (Result<Downloadable, _>, bool) = match &mod_.identifier {
            ModIdentifier::CurseForgeProject(project_id) => {
                let result = upgrade::curseforge(
                    curseforge,
                    *project_id,
                    &profile.game_version,
                    &profile.mod_loader,
                    mod_.check_game_version,
                    mod_.check_mod_loader,
                )
                .await;
                if matches!(result, Err(upgrade::Error::NoCompatibleFile))
                    && profile.mod_loader == config::structs::ModLoader::Quilt
                {
                    (
                        upgrade::curseforge(
                            curseforge,
                            *project_id,
                            &profile.game_version,
                            &config::structs::ModLoader::Fabric,
                            mod_.check_game_version,
                            mod_.check_mod_loader,
                        )
                        .await
                        .map(Into::into),
                        true,
                    )
                } else {
                    (result.map(Into::into), false)
                }
            },
            ModIdentifier::ModrinthProject(project_id) => {
                let result = upgrade::modrinth(
                    modrinth,
                    project_id,
                    &profile.game_version,
                    &profile.mod_loader,
                    mod_.check_game_version,
                    mod_.check_mod_loader,
                )
                .await;
                if matches!(result, Err(upgrade::Error::NoCompatibleFile))
                    && profile.mod_loader == config::structs::ModLoader::Quilt
                {
                    (
                        upgrade::modrinth(
                            modrinth,
                            project_id,
                            &profile.game_version,
                            &config::structs::ModLoader::Fabric,
                            mod_.check_game_version,
                            mod_.check_mod_loader,
                        )
                        .await
                        .map(Into::into),
                        true,
                    )
                } else {
                    (result.map(Into::into), false)
                }
            },
            ModIdentifier::GitHubRepository(full_name) => {
                let result = upgrade::github(
                    &github.repos(&full_name.0, &full_name.1),
                    &profile.game_version,
                    &profile.mod_loader,
                    mod_.check_game_version,
                    mod_.check_mod_loader,
                )
                .await;
                if matches!(result, Err(upgrade::Error::NoCompatibleFile))
                    && profile.mod_loader == config::structs::ModLoader::Quilt
                {
                    (
                        upgrade::github(
                            &github.repos(&full_name.0, &full_name.1),
                            &profile.game_version,
                            &config::structs::ModLoader::Fabric,
                            mod_.check_game_version,
                            mod_.check_mod_loader,
                        )
                        .await
                        .map(Into::into),
                        true,
                    )
                } else {
                    (result.map(Into::into), false)
                }
            },
        };

        match result {
            Ok(result) => {
                println!(
                    "{} {:40}{}",
                    if backwards_compat {
                        backwards_compat_msg = true;
                        YELLOW_TICK.clone()
                    } else {
                        TICK.clone()
                    },
                    mod_.name,
                    format!("({})", result.filename).dimmed()
                );
                to_download.push(result);
            },
            Err(err) => {
                eprintln!("{}", format!("{} {:40}{}", CROSS, mod_.name, err).red());
                error = true;
            },
        }
    }
    if backwards_compat_msg {
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

    println!("\n{}\n", "Downloading Mod Files".bold());

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

    for downloadable in to_download {
        eprint!("Downloading {}... ", downloadable.filename.dimmed());
        let contents = reqwest::get(downloadable.download_url)
            .await?
            .bytes()
            .await?;
        upgrade::write_mod_file(profile, contents, &downloadable.filename).await?;
        println!("{}", &*TICK);
    }
    for installable in to_install {
        eprint!(
            "Installing  {}... ",
            installable.0.to_string_lossy().dimmed()
        );
        copy(installable.1, profile.output_dir.join(installable.0)).await?;
        println!("{}", &*TICK);
    }

    if error {
        bail!("\nCould not get the latest compatible version of some mods")
    }

    Ok(())
}
