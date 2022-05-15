use crate::{download::download, CROSS, STYLE, TICK};
use anyhow::{bail, Result};
use colored::Colorize;
use fs_extra::file::{move_file, CopyOptions};
use furse::Furse;
use indicatif::{ProgressBar, ProgressStyle};
use libium::{
    config::structs::{Modpack, ModpackIdentifier},
    mutex_ext::MutexExt,
    upgrade::{modpack_downloadable::get_curseforge_manifest, Downloadable},
};
use std::{
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
            println!("{}", "Getting modpack manifest... ".bold());
            let progress_bar =
                ProgressBar::new(0).with_style(ProgressStyle::default_bar().template(
                    "{spinner} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
                )
                .progress_chars("#>-"));
            progress_bar.enable_steady_tick(100);
            let manifest =
                get_curseforge_manifest(curseforge.clone(), project_id, |additional, total| {
                    progress_bar.set_position(progress_bar.position() + additional as u64);
                    if progress_bar.length() == 0 {
                        progress_bar.set_length(total);
                    }
                })
                .await?;
            progress_bar.finish_and_clear();

            let to_download = Arc::new(Mutex::new(Vec::new()));
            let progress_bar = Arc::new(Mutex::new(
                ProgressBar::new(manifest.files.len() as u64).with_style(STYLE.clone()),
            ));
            let mut tasks = Vec::new();
            let error = Arc::new(AtomicBool::new(false));

            println!("{}\n", "Determining Files to Download".bold());
            {
                progress_bar.force_lock().enable_steady_tick(100);
            }
            for file in &manifest.files {
                let to_download = to_download.clone();
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
                            progress_bar.println(format!("{} {}", &*TICK, downloadable.filename));
                            {
                                let mut to_download = to_download.force_lock();
                                to_download.push(downloadable);
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
            let mut to_download = Arc::try_unwrap(to_download)
                .expect("Failed to run threads to completion")
                .into_inner()?;

            create_dir_all(modpack.output_dir.join("mods").join(".old")).await?;
            for file in read_dir(&modpack.output_dir.join("mods"))? {
                let file = file?;
                if file.file_type()?.is_file() {
                    let filename = file.file_name();
                    let filename = filename.to_str().unwrap();
                    if let Some(index) = to_download
                        .iter()
                        .position(|thing: &Downloadable| filename == thing.filename)
                    {
                        to_download.swap_remove(index);
                    } else if move_file(
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
            download(
                Arc::new(modpack.output_dir.join("mods")),
                to_download,
                Vec::new(),
            )
            .await?;

            if error.load(Ordering::Relaxed) {
                bail!("\nCould not get download some mods")
            }
        },
    }
    Ok(())
}
