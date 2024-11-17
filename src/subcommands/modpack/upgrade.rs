use crate::{
    download::{clean, download, read_overrides},
    STYLE_BYTE, TICK,
};
use anyhow::{Context as _, Result};
use colored::Colorize as _;
use indicatif::ProgressBar;
use libium::{
    config::structs::{Modpack, ModpackIdentifier},
    iter_ext::IterExt as _,
    modpack::{
        curseforge::structs::Manifest as CFManifest, modrinth::structs::Metadata as MRMetadata,
        read_file_from_zip, zip_extract,
    },
    upgrade::{from_modpack_file, try_from_cf_file, DistributionDeniedError, DownloadData},
    CURSEFORGE_API, HOME,
};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::task::JoinSet;

pub async fn upgrade(modpack: &'_ Modpack) -> Result<()> {
    let mut to_download: Vec<DownloadData> = Vec::new();
    let mut to_install = Vec::new();
    let install_msg;

    let progress_bar = ProgressBar::new(0).with_style(STYLE_BYTE.clone());
    let modpack_filepath = modpack
        .identifier
        .download_file(
            |total| {
                progress_bar.println("Downloading Modpack".bold().to_string());
                progress_bar.enable_steady_tick(Duration::from_millis(100));
                progress_bar.set_length(total as u64);
            },
            |additional| {
                progress_bar.inc(additional as u64);
            },
        )
        .await?;
    let modpack_file = File::open(&modpack_filepath)?;
    progress_bar.finish_and_clear();

    match &modpack.identifier {
        ModpackIdentifier::CurseForgeModpack(_) => {
            let manifest: CFManifest = serde_json::from_str(
                &read_file_from_zip(BufReader::new(modpack_file), "manifest.json")?
                    .context("Does not contain manifest")?,
            )?;

            eprint!("\n{}", "Determining files to download... ".bold());

            let file_ids = manifest.files.iter().map(|file| file.file_id).collect();
            let files = CURSEFORGE_API.get_files(file_ids).await?;
            println!("{} Fetched {} mods", &*TICK, files.len());

            let mut tasks = JoinSet::new();
            let mut msg_shown = false;
            for file in files {
                match try_from_cf_file(file) {
                    Ok((_, mut downloadable)) => {
                        downloadable.output = PathBuf::from(
                            if Path::new(&downloadable.filename())
                                .extension()
                                .is_some_and(|ext| ext.eq_ignore_ascii_case(".zip"))
                            {
                                "resourcepacks"
                            } else {
                                "mods"
                            },
                        )
                        .join(downloadable.filename());
                        to_download.push(downloadable);
                    }
                    Err(DistributionDeniedError(mod_id, file_id)) => {
                        if !msg_shown {
                            println!("\n{}", "The following mod(s) have denied 3rd parties such as Ferium from downloading it".red().bold());
                        }
                        msg_shown = true;
                        tasks.spawn(async move {
                            let project = CURSEFORGE_API.get_mod(mod_id).await?;
                            eprintln!(
                                "- {}
                           \r  {}",
                                project.name.bold(),
                                format!("{}/download/{file_id}", project.links.website_url)
                                    .blue()
                                    .underline(),
                            );
                            Ok::<(), furse::Error>(())
                        });
                    }
                }
            }

            for res in tasks.join_all().await {
                res?;
            }

            install_msg = format!(
                "You can play this modpack using Minecraft {} with {}",
                manifest.minecraft.version,
                manifest
                    .minecraft
                    .mod_loaders
                    .iter()
                    .map(|this| &this.id)
                    .display(", ")
            );

            if modpack.install_overrides {
                let tmp_dir = HOME
                    .join(".config")
                    .join("ferium")
                    .join(".tmp")
                    .join(manifest.name);
                zip_extract(&modpack_filepath, &tmp_dir)?;
                to_install = read_overrides(&tmp_dir.join(manifest.overrides))?;
            }
        }
        ModpackIdentifier::ModrinthModpack(_) => {
            let metadata: MRMetadata = serde_json::from_str(
                &read_file_from_zip(BufReader::new(modpack_file), "modrinth.index.json")?
                    .context("Does not contain metadata file")?,
            )?;

            for file in metadata.files {
                to_download.push(from_modpack_file(file));
            }

            install_msg = format!(
                "You can play this modpack using the following:\n{}",
                metadata
                    .dependencies
                    .iter()
                    .map(|this| format!("{:?} {}", this.0, this.1))
                    .display("\n")
            );

            if modpack.install_overrides {
                let tmp_dir = HOME
                    .join(".config")
                    .join("ferium")
                    .join(".tmp")
                    .join(metadata.name);
                zip_extract(&modpack_filepath, &tmp_dir)?;
                to_install = read_overrides(&tmp_dir.join("overrides"))?;
            }
        }
    }
    clean(
        &modpack.output_dir.join("mods"),
        &mut to_download,
        &mut Vec::new(),
    )
    .await?;
    clean(
        &modpack.output_dir.join("resourcepacks"),
        &mut to_download,
        &mut Vec::new(),
    )
    .await?;
    // TODO: Check for `to_install` files that are already installed
    if to_download.is_empty() && to_install.is_empty() {
        println!("\n{}", "All up to date!".bold());
    } else {
        println!(
            "\n{}\n",
            format!("Downloading {} Mod Files", to_download.len()).bold()
        );
        download(modpack.output_dir.clone(), to_download, to_install).await?;
    }
    println!("\n{}", install_msg.bold());
    Ok(())
}
