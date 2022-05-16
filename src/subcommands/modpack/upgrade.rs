use crate::{download::download, CROSS, STYLE_BYTE, STYLE_NO, TICK};
use anyhow::{bail, Result};
use colored::Colorize;
use fs_extra::file::{move_file, CopyOptions};
use furse::Furse;
use indicatif::ProgressBar;
use libium::{
    config::structs::{Modpack, ModpackIdentifier},
    mutex_ext::MutexExt,
    upgrade::{modpack_downloadable::get_curseforge_manifest, Downloadable},
};
use std::{
    ffi::OsStr,
    fs::read_dir,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tokio::{
    fs::{create_dir_all, remove_file},
    spawn,
};

pub async fn upgrade(
    // modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    modpack: &Modpack,
) -> Result<()> {
    match modpack.identifier {
        ModpackIdentifier::CurseForgeModpack(project_id) => {
            println!("{}", "Getting modpack manifest".bold());
            let progress_bar = ProgressBar::new(0).with_style(STYLE_BYTE.clone());
            progress_bar.enable_steady_tick(100);
            let manifest = get_curseforge_manifest(
                curseforge.clone(),
                project_id,
                |total| progress_bar.set_length(total),
                |additional| progress_bar.set_position(progress_bar.position() + additional as u64),
            )
            .await?;
            progress_bar.finish_and_clear();

            let respacks_to_download = Arc::new(Mutex::new(Vec::new()));
            let mods_to_download = Arc::new(Mutex::new(Vec::new()));
            let progress_bar = Arc::new(Mutex::new(
                ProgressBar::new(manifest.files.len() as u64).with_style(STYLE_NO.clone()),
            ));
            let mut tasks = Vec::new();
            let error = Arc::new(AtomicBool::new(false));

            println!("{}\n", "Determining Files to Download".bold());
            {
                progress_bar.force_lock().enable_steady_tick(100);
            }
            for file in &manifest.files {
                let respacks_to_download = respacks_to_download.clone();
                let mods_to_download = mods_to_download.clone();
                let progress_bar = progress_bar.clone();
                let curseforge = curseforge.clone();
                let error = error.clone();
                let file = file.clone();
                tasks.push(spawn(async move {
                    let result =
                        Downloadable::from_ids(curseforge, file.project_id, file.file_id).await;
                    let progress_bar = progress_bar.force_lock();
                    match result {
                        Ok(downloadable) => {
                            if downloadable.filename.ends_with("jar") {
                                progress_bar
                                    .println(format!("{} {}", &*TICK, downloadable.filename));
                                mods_to_download.force_lock().push(downloadable);
                            } else if downloadable.filename.ends_with("zip") {
                                progress_bar
                                    .println(format!("{} {}", &*TICK, downloadable.filename));
                                respacks_to_download.force_lock().push(downloadable);
                            } else {
                                progress_bar.println(format!(
                                    "{}",
                                    format!(
                                        "{} {:6} of {:6} Cannot identify file extension",
                                        CROSS, file.file_id, file.project_id,
                                    )
                                    .red()
                                ));
                            }
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
                            error.store(true, Ordering::Relaxed);
                        },
                    }
                    progress_bar.set_position(progress_bar.position() + 1);
                }));
            }
            for handle in tasks {
                handle.await?;
            }
            Arc::try_unwrap(progress_bar)
                .expect("Failed to run threads to completion")
                .into_inner()?
                .finish_and_clear();
            let mut mods_to_download = Arc::try_unwrap(mods_to_download)
                .expect("Failed to run threads to completion")
                .into_inner()?;
            let mut respacks_to_download = Arc::try_unwrap(respacks_to_download)
                .expect("Failed to run threads to completion")
                .into_inner()?;

            create_dir_all(modpack.output_dir.join("mods").join(".old")).await?;
            for file in read_dir(&modpack.output_dir.join("mods"))? {
                let file = file?;
                if file.file_type()?.is_file() {
                    let filename = file.file_name();
                    let filename = filename.to_str().unwrap();
                    if let Some(index) = mods_to_download
                        .iter()
                        .position(|thing: &Downloadable| filename == thing.filename)
                    {
                        mods_to_download.swap_remove(index);
                    } else if file.path().extension() == Some(&OsStr::new("part"))
                        || move_file(
                            file.path(),
                            modpack.output_dir.join("mods").join(".old").join(filename),
                            &CopyOptions::new(),
                        )
                        .is_err()
                    {
                        remove_file(file.path()).await?;
                    }
                }
            }
            create_dir_all(modpack.output_dir.join("resourcepacks").join(".old")).await?;
            for file in read_dir(&modpack.output_dir.join("resourcepacks"))? {
                let file = file?;
                if file.file_type()?.is_file() {
                    let filename = file.file_name();
                    let filename = filename.to_str().unwrap();
                    if let Some(index) = respacks_to_download
                        .iter()
                        .position(|thing: &Downloadable| filename == thing.filename)
                    {
                        respacks_to_download.swap_remove(index);
                    } else if file.path().extension() == Some(&OsStr::new("part"))
                        || move_file(
                            file.path(),
                            modpack
                                .output_dir
                                .join("resourcepacks")
                                .join(".old")
                                .join(filename),
                            &CopyOptions::new(),
                        )
                        .is_err()
                    {
                        remove_file(file.path()).await?;
                    }
                }
            }
            if mods_to_download.is_empty() && respacks_to_download.is_empty() {
                println!("\n{}", "All up to date!".bold());
            } else {
                if !mods_to_download.is_empty() {
                    println!("\n{}\n", "Downloading Mod Files".bold());
                    download(
                        Arc::new(modpack.output_dir.join("mods")),
                        mods_to_download,
                        Vec::new(),
                    )
                    .await?;
                }

                if !respacks_to_download.is_empty() {
                    println!("\n{}\n", "Downloading Resource Pack Files".bold());
                    download(
                        Arc::new(modpack.output_dir.join("resourcepacks")),
                        respacks_to_download,
                        Vec::new(),
                    )
                    .await?;
                }
            }

            if error.load(Ordering::Relaxed) {
                bail!("\nCould not get download some mods")
            }
        },
    }
    Ok(())
}
