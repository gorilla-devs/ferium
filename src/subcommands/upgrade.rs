use crate::{CROSS, TICK, YELLOW_TICK};
use anyhow::{bail, Result};
use colored::Colorize;
use ferinth::Ferinth;
use furse::Furse;
use itertools::Itertools;
use libium::config;
use libium::upgrade;
use octocrab::Octocrab;
use tokio::fs::remove_file;

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
impl From<ferinth::structures::version_structs::VersionFile> for Downloadable {
    fn from(file: ferinth::structures::version_structs::VersionFile) -> Self {
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
    let mut backwards_compat = false;
    let mut error = false;

    println!("{}\n", "Determining the Latest Compatible Versions".bold());
    for mod_ in &profile.mods {
        use libium::config::structs::ModIdentifier;
        let result: Result<(Downloadable, bool), _> = match &mod_.identifier {
            ModIdentifier::CurseForgeProject(project_id) => upgrade::curseforge(
                curseforge,
                profile,
                *project_id,
                mod_.check_game_version,
                mod_.check_mod_loader,
            )
            .await
            .map(|ok| (ok.0.into(), ok.1)),
            ModIdentifier::ModrinthProject(project_id) => upgrade::modrinth(
                modrinth,
                profile,
                project_id,
                mod_.check_game_version,
                mod_.check_mod_loader,
            )
            .await
            .map(|ok| {
                let version = ok.0;
                for file in &version.files {
                    if file.primary {
                        return (file.clone().into(), ok.1);
                    }
                }
                (version.files[0].clone().into(), ok.1)
            }),
            ModIdentifier::GitHubRepository(full_name) => upgrade::github(
                &github.repos(&full_name.0, &full_name.1),
                profile,
                mod_.check_game_version,
                mod_.check_mod_loader,
            )
            .await
            .map(|ok| (ok.0.into(), ok.1)),
        };
        match result {
            Ok(result) => {
                println!(
                    "{} {:40}{}",
                    if result.1 {
                        backwards_compat = true;
                        YELLOW_TICK.clone()
                    } else {
                        TICK.clone()
                    },
                    mod_.name,
                    format!("({})", result.0.filename).dimmed()
                );
                to_download.push(result);
            },
            Err(err) => {
                eprintln!("{}", format!("{} {:40}{}", CROSS, mod_.name, err).red());
                error = true;
            },
        }
    }
    if backwards_compat {
        println!(
            "{}",
            "Fabric mod using Quilt backwards compatibility".yellow()
        );
    }

    eprint!("\n{}", "Downloading Mod Files... ".bold());
    for file in std::fs::read_dir(&profile.output_dir)? {
        let file = file?;
        let path = file.path();
        if path.is_file() {
            let mut index = None;
            // If a file is already downloaded
            if let Some(downloadable) = to_download
                .iter()
                .find_position(|thing| file.file_name().to_str().unwrap() == thing.0.filename)
            {
                index = Some(downloadable.0);
            }
            match index {
                // Then don't download the file
                Some(index) => {
                    to_download.swap_remove(index);
                },
                // Or else delete the file
                None => remove_file(path).await?,
            }
        }
    }
    match {
        for downloadable in to_download {
            let contents = reqwest::get(downloadable.0.download_url)
                .await?
                .bytes()
                .await?;
            upgrade::write_mod_file(profile, contents, &downloadable.0.filename).await?;
        }
        Ok::<(), anyhow::Error>(())
    } {
        Ok(_) => println!("{}", *TICK),
        Err(_) => bail!("{}", CROSS),
    }

    if error {
        bail!("\nCould not get the latest compatible version of some mods")
    }

    Ok(())
}
