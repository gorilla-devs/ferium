use crate::{
    download::{clean, download},
    subcommands::upgrade::{self, PlatformDownloadable},
    TICK,
};
use anyhow::{bail, Result};
use colored::Colorize;
use ferinth::{structures::version::Hashes, Ferinth};
use furse::Furse;
use libium::{
    config::structs::{ModLoader, Profile},
    modpack,
    upgrade::Downloadable,
    HOME,
};
use octocrab::Octocrab;
use sha1::Sha1;
use sha2::{Digest, Sha512};
use std::{collections::HashMap, fs::read_dir, path::PathBuf};
use tokio::fs::{read, remove_file};

#[allow(clippy::too_many_arguments)]
pub async fn modrinth(
    modrinth: Ferinth,
    curseforge: Furse,
    github: Octocrab,
    profile: &Profile,
    version_id: String,
    summary: Option<String>,
    out_dir: PathBuf,
    overrides: Option<PathBuf>,
    mod_loader_version: String,
) -> Result<()> {
    use modpack::modrinth::{
        create,
        structs::{DependencyID, Game, Metadata, ModpackFile},
    };
    println!(); // TODO: remove after proper formatting
    let output = out_dir.join(format!("{}-{version_id}.mrpack", profile.name));

    if let Some(overrides) = &overrides {
        if overrides.join("mods").exists() {
            for entry in read_dir(overrides.join("mods"))? {
                if !entry?.file_name().to_string_lossy().starts_with('.') {
                    bail!("Your overrides directory's mods folder contains files in it, these will interfere with ferium's modpack packaging process! Use the user folder of your profile's output directory to manually include mod files")
                }
            }
        }
    }

    let (platform_downloadables, error) =
        upgrade::get_platform_downloadables(modrinth, curseforge, github, profile).await?;

    let mut files = Vec::new();
    let mut to_download = Vec::<Downloadable>::new();
    let mut calculate_hashes = Vec::new();
    let mut to_install = Vec::new();

    for downloadable in platform_downloadables {
        match downloadable {
            PlatformDownloadable::Modrinth(version_file, _) => files.push(ModpackFile {
                path: PathBuf::from("mods").join(version_file.filename),
                hashes: version_file.hashes,
                env: None,
                downloads: vec![version_file.url],
                file_size: version_file.size,
            }),
            PlatformDownloadable::CurseForge(file) => to_download.push(file.try_into()?),
            PlatformDownloadable::GitHub(asset) => {
                calculate_hashes.push(asset.clone());
                to_download.push(asset.into());
            },
        }
    }

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

    to_download
        .iter_mut()
        // Download directly to the output directory
        .map(|thing| thing.output = thing.filename().into())
        .for_each(drop); // Doesn't drop any data, just runs the interator
    let included_mods = HOME
        .join(".config")
        .join("ferium")
        .join(".tmp")
        .join("export")
        .join(&profile.name);
    clean(&included_mods, &mut to_download, &mut to_install).await?;
    if !to_download.is_empty() || !to_install.is_empty() {
        println!("\n{}", "Downloading non-Modrinth files".bold());
        download(included_mods.clone(), to_download, to_install).await?;
    }

    for asset in calculate_hashes {
        let path = included_mods.join(&asset.name);
        if path.is_file() {
            let data = read(&path).await?;
            remove_file(&path).await?;
            files.push(ModpackFile {
                path: PathBuf::from("mods").join(asset.name),
                hashes: Hashes {
                    sha512: format!("{:x}", Sha512::digest(&data)),
                    sha1: format!("{:x}", Sha1::digest(data)),
                },
                env: None,
                downloads: vec![asset.browser_download_url],
                file_size: asset.size as usize,
            });
        }
    }

    let mut dependencies = HashMap::new();
    dependencies.insert(DependencyID::Minecraft, profile.game_version.clone());
    dependencies.insert(DependencyID::QuiltLoader, mod_loader_version);
    let metadata = serde_json::to_string_pretty(&Metadata {
        format_version: 1,
        game: Game::Minecraft,
        version_id,
        name: profile.name.clone(),
        summary,
        files,
        dependencies,
    })?;

    eprint!("\n{}", "Creating modpack file... ".bold());
    create(
        &output,
        &metadata,
        overrides.as_deref(),
        Some(&included_mods),
    )
    .await?;
    println!("{} {}", *TICK, output.display().to_string().dimmed());

    if error {
        bail!("\nCould not get the latest compatible version of some mods")
    }
    Ok(())
}
