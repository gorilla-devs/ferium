use crate::{
    download::{clean, download},
    CROSS, STYLE_BYTE, STYLE_NO, TICK,
};
use anyhow::Result;
use colored::Colorize;
use ferinth::Ferinth;
use furse::Furse;
use indicatif::ProgressBar;
use itertools::Itertools;
use libium::{
    config::structs::{Modpack, ModpackIdentifier},
    mutex_ext::MutexExt,
    upgrade::{
        modpack_downloadable::{get_curseforge_manifest, get_modrinth_manifest},
        Downloadable,
    },
};
use std::sync::{Arc, Mutex};
use tokio::spawn;

pub async fn upgrade(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    modpack: &'_ Modpack,
) -> Result<()> {
    let mut to_download = Vec::new();
    let install_msg;
    match &modpack.identifier {
        ModpackIdentifier::CurseForgeModpack(project_id) => {
            to_download = {
                println!("{}", "Getting modpack manifest".bold());
                let progress_bar = ProgressBar::new(0).with_style(STYLE_BYTE.clone());
                progress_bar.enable_steady_tick(100);
                let manifest = get_curseforge_manifest(
                    curseforge.clone(),
                    *project_id,
                    |total| progress_bar.set_length(total),
                    |additional| {
                        progress_bar.set_position(progress_bar.position() + additional as u64);
                    },
                )
                .await?;
                progress_bar.finish_and_clear();

                println!("\n{}\n", "Determining Files to Download".bold());
                let progress_bar = Arc::new(Mutex::new(
                    ProgressBar::new(manifest.files.len() as u64).with_style(STYLE_NO.clone()),
                ));
                progress_bar.force_lock().enable_steady_tick(100);
                let mut tasks = Vec::new();
                let to_download = Arc::new(Mutex::new(to_download));

                for file in &manifest.files {
                    let to_download = to_download.clone();
                    let progress_bar = progress_bar.clone();
                    let curseforge = curseforge.clone();
                    let file = file.clone();
                    tasks.push(spawn(async move {
                        let result =
                            Downloadable::from_file_id(curseforge, file.project_id, file.file_id)
                                .await;
                        let progress_bar = progress_bar.force_lock();
                        match result {
                            Ok(downloadable) => {
                                progress_bar.println(format!(
                                    "{} {}",
                                    &*TICK,
                                    downloadable.filename()
                                ));
                                to_download.force_lock().push(downloadable);
                            },
                            Err(err) => {
                                progress_bar.println(format!(
                                    "{}",
                                    format!(
                                        "{} {:6} of {:6} {}",
                                        CROSS, file.file_id, file.project_id, err
                                    )
                                    .red()
                                ));
                            },
                        }
                        progress_bar.set_position(progress_bar.position() + 1);
                    }));
                }
                for handle in tasks {
                    handle.await?;
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

                Arc::try_unwrap(progress_bar)
                    .expect("Failed to run threads to completion")
                    .into_inner()?
                    .finish_and_clear();
                Arc::try_unwrap(to_download)
                    .expect("Failed to run threads to completion")
                    .into_inner()?
            }
        },
        ModpackIdentifier::ModrinthModpack(project_id) => {
            eprint!("Getting modpack metadata... ");
            let metadata =
                get_modrinth_manifest(modrinth.clone(), project_id, |_| (), |_| ()).await?;
            println!("{}", &*CROSS);

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
    if to_download.is_empty() {
        println!("\n{}", "All up to date!".bold());
    } else {
        println!(
            "\n{}\n",
            format!("Downloading {} Mod Files", to_download.len()).bold()
        );
        download(
            Arc::new(modpack.output_dir.clone()),
            to_download,
            Vec::new(),
        )
        .await?;
    }
    println!("\n{}", install_msg.bold());
    Ok(())
}
