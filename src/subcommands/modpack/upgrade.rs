use crate::{
    download::{clean, download, read_overrides},
    style_byte, TICK,
};
use anyhow::Result;
use colored::Colorize;
use ferinth::Ferinth;
use furse::Furse;
use indicatif::ProgressBar;
use itertools::Itertools;
use libium::{
    config::structs::{Modpack, ModpackIdentifier},
    modpack::{
        curseforge::{deser_manifest, read_manifest_file},
        extract_zip,
        modrinth::{deser_metadata, read_metadata_file},
    },
    upgrade::{
        modpack_downloadable::{download_curseforge_modpack, download_modrinth_modpack},
        DistributionDeniedError, Downloadable,
    },
    HOME,
};
use std::{sync::Arc, time::Duration};
use tokio::spawn;

#[allow(clippy::future_not_send)] // 3rd party library doesn't implement `Send`
pub async fn upgrade(modrinth: &Ferinth, curseforge: &Furse, modpack: &'_ Modpack) -> Result<()> {
    let mut to_download: Vec<Downloadable> = Vec::new();
    let mut to_install = Vec::new();
    let install_msg;
    match &modpack.identifier {
        ModpackIdentifier::CurseForgeModpack(project_id) => {
            println!("{}", "Downloading Modpack".bold());
            let progress_bar = ProgressBar::new(0).with_style(style_byte());
            let modpack_file = download_curseforge_modpack(
                &curseforge.clone(),
                *project_id,
                |total| {
                    progress_bar.enable_steady_tick(Duration::from_millis(100));
                    progress_bar.set_length(total);
                },
                |additional| {
                    progress_bar.inc(additional as u64);
                },
            )
            .await?;
            let manifest = deser_manifest(&read_manifest_file(&modpack_file)?)?;
            progress_bar.finish_and_clear();

            eprint!("\n{}", "Determining files to download... ".bold());

            let file_ids = manifest.files.iter().map(|file| file.file_id).collect();
            let files = curseforge.get_files(file_ids).await?;
            println!("{} Fetched {} mods", &*TICK, files.len());

            let mut tasks = Vec::new();
            let mut msg_shown = false;
            for file in files {
                match file.try_into() {
                    Ok(downloadable) => {
                        to_download.push(downloadable);
                    },
                    Err(DistributionDeniedError(mod_id, file_id)) => {
                        if !msg_shown {
                            println!("\n{}", "The following mod(s) have denied 3rd parties such as Ferium from downloading it".red().bold());
                        }
                        msg_shown = true;
                        let curseforge = curseforge.clone();
                        tasks.push(spawn(async move {
                            let project = curseforge.get_mod(mod_id).await?;
                            eprintln!(
                                "- {}
                           \r  {}",
                                project.name.bold(),
                                format!("{}/download/{file_id}", project.links.website_url)
                                    .blue()
                                    .underline(),
                            );
                            Ok::<(), furse::Error>(())
                        }));
                    },
                }
            }

            for task in tasks {
                task.await??;
            }

            install_msg = format!(
                "You can play this modpack using Minecraft {} with {}",
                manifest.minecraft.version,
                manifest
                    .minecraft
                    .mod_loaders
                    .iter()
                    .map(|this| &this.id)
                    .format(", or ")
            );

            if modpack.install_overrides {
                let tmp_dir = HOME
                    .join(".config")
                    .join("ferium")
                    .join(".tmp")
                    .join(manifest.name);
                extract_zip(modpack_file, &tmp_dir).await?;
                to_install = read_overrides(&tmp_dir.join(manifest.overrides))?;
            }
        },
        ModpackIdentifier::ModrinthModpack(project_id) => {
            println!("{}", "Downloading Modpack".bold());
            let progress_bar = ProgressBar::new(0).with_style(style_byte());
            let modpack_file = download_modrinth_modpack(
                &modrinth.clone(),
                project_id,
                |total| {
                    progress_bar.enable_steady_tick(Duration::from_millis(100));
                    progress_bar.set_length(total);
                },
                |additional| {
                    progress_bar.inc(additional as u64);
                },
            )
            .await?;
            let metadata = deser_metadata(&read_metadata_file(&modpack_file)?)?;
            progress_bar.finish_and_clear();

            for file in metadata.files {
                to_download.push(file.into());
            }

            install_msg = format!(
                "You can play this modpack using the following:\n{}",
                metadata
                    .dependencies
                    .iter()
                    .map(|this| format!("{:?} {}", this.0, this.1))
                    .format("\n")
            );

            if modpack.install_overrides {
                let tmp_dir = HOME
                    .join(".config")
                    .join("ferium")
                    .join(".tmp")
                    .join(metadata.name);
                extract_zip(modpack_file, &tmp_dir).await?;
                to_install = read_overrides(&tmp_dir.join("overrides"))?;
            }
        },
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
    if to_download.is_empty() && to_install.is_empty() {
        println!("\n{}", "All up to date!".bold());
    } else {
        println!(
            "\n{}\n",
            format!("Downloading {} Mod Files", to_download.len()).bold()
        );
        download(
            Arc::new(modpack.output_dir.clone()),
            to_download,
            to_install,
        )
        .await?;
    }
    println!("\n{}", install_msg.bold());
    Ok(())
}
