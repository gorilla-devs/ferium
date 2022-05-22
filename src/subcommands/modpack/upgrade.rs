use crate::{
    download::{clean, download, read_overrides},
    CROSS, STYLE_BYTE, TICK,
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
        extract_modpack,
        modrinth::{deser_metadata, read_metadata_file},
    },
    upgrade::{
        modpack_downloadable::{download_curseforge_modpack, download_modrinth_modpack},
        Downloadable,
    },
    HOME,
};
use std::sync::Arc;
use tokio::spawn;

#[allow(clippy::future_not_send)] // 3rd party library doesn't implement `Send`
pub async fn upgrade(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    modpack: &'_ Modpack,
) -> Result<()> {
    let mut to_download: Vec<Downloadable> = Vec::new();
    let mut to_install = Vec::new();
    let install_msg;
    match &modpack.identifier {
        ModpackIdentifier::CurseForgeModpack(project_id) => {
            println!("{}", "Downloading Modpack".bold());
            let progress_bar = ProgressBar::new(0).with_style(STYLE_BYTE.clone());
            let modpack_file = download_curseforge_modpack(
                curseforge.clone(),
                *project_id,
                |total| {
                    progress_bar.enable_steady_tick(100);
                    progress_bar.set_length(total);
                },
                |additional| {
                    progress_bar.set_position(progress_bar.position() + additional as u64);
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
            for file in files {
                let mod_id = file.mod_id;
                let file_id = file.id;
                match file.try_into() {
                    Ok(downloadable) => to_download.push(downloadable),
                    Err(_) => {
                        let curseforge = curseforge.clone();
                        tasks.push(spawn(async move {
                            let project = curseforge.get_mod(mod_id).await?;
                            eprintln!(
                                "\n{}\n  Manual download: {}",
                                format!(
                                "{} {} has denied third parties like Ferium from downloading it",
                                CROSS, project.name
                            )
                                .red()
                                .bold(),
                                format!("{}/files/{}", project.links.website_url, file_id)
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
                extract_modpack(modpack_file, &tmp_dir).await?;
                to_install = read_overrides(&tmp_dir.join(manifest.overrides))?;
            }
        },
        ModpackIdentifier::ModrinthModpack(project_id) => {
            println!("{}", "Downloading Modpack".bold());
            let progress_bar = ProgressBar::new(0).with_style(STYLE_BYTE.clone());
            let modpack_file = download_modrinth_modpack(
                modrinth.clone(),
                project_id,
                |total| {
                    progress_bar.enable_steady_tick(100);
                    progress_bar.set_length(total);
                },
                |additional| {
                    progress_bar.set_position(progress_bar.position() + additional as u64);
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
                extract_modpack(modpack_file, &tmp_dir).await?;
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
