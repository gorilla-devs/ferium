use crate::{STYLE_BYTE, TICK};
use anyhow::{anyhow, bail, Error, Result};
use colored::Colorize;
use fs_extra::{
    dir::{copy as copy_dir, CopyOptions as DirCopyOptions},
    file::{move_file, CopyOptions as FileCopyOptions},
};
use indicatif::ProgressBar;
use libium::{mutex_ext::MutexExt, upgrade::Downloadable};
use std::{
    ffi::OsString,
    fs::read_dir,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::{
    fs::{copy, create_dir_all, remove_file},
    spawn,
    sync::Semaphore,
};

/// Check the given `directory`
///
/// - If there are files there that are not in `to_download` or `to_install`, they will be moved to `directory`/.old
/// - If a file in `to_download` or `to_install` is already there, it will be removed from the respective vector
/// - If the file is a `.part` file or if the move failed, the file will be deleted
pub async fn clean(
    directory: &Path,
    to_download: &mut Vec<Downloadable>,
    to_install: &mut Vec<(OsString, PathBuf)>,
) -> Result<()> {
    let len = to_download.len();
    to_download.sort_unstable_by_key(|downloadable| downloadable.output.clone());
    to_download.dedup_by_key(|downloadable| downloadable.output.clone());
    if to_download.len() < len {
        println!(
            "{}",
            format!(
                "Warning: {} duplicate files were found, please check your mods for duplicates!",
                len - to_download.len()
            )
            .yellow()
            .bold()
        );
    }
    create_dir_all(directory.join(".old")).await?;
    for file in read_dir(&directory)? {
        let file = file?;
        // If it's a file
        if file.file_type()?.is_file() {
            let filename = file.file_name();
            let filename = filename.to_str().unwrap();
            // If it is already downloaded
            if let Some(index) = to_download
                .iter()
                .position(|thing| filename == thing.filename())
            {
                // Don't download it
                to_download.swap_remove(index);
            // Likewise, if it is already installed
            } else if let Some(index) = to_install.iter().position(|thing| filename == thing.0) {
                // Don't install it
                to_install.swap_remove(index);
            // Or else, move the file to `directory`/.old
            // If the file is a `.part` file or if the move failed, delete the file
            } else if filename.ends_with("part")
                || move_file(
                    file.path(),
                    directory.join(".old").join(filename),
                    &FileCopyOptions::new(),
                )
                .is_err()
            {
                remove_file(file.path()).await?;
            }
        }
    }
    Ok(())
}

/// Construct a `to_install` vector from the `directory`
pub fn read_overrides(directory: &Path) -> Result<Vec<(OsString, PathBuf)>> {
    let mut to_install = Vec::new();
    for file in read_dir(directory)? {
        let file = file?;
        to_install.push((file.file_name(), file.path()));
    }
    Ok(to_install)
}

/// Download and install the files in `to_download` and `to_install` to `output_dir`
pub async fn download(
    output_dir: Arc<PathBuf>,
    to_download: Vec<Downloadable>,
    to_install: Vec<(OsString, PathBuf)>,
) -> Result<()> {
    create_dir_all(&*output_dir).await?;
    let progress_bar = Arc::new(Mutex::new(
        ProgressBar::new(to_download.len() as u64).with_style(STYLE_BYTE.clone()),
    ));
    progress_bar.force_lock().enable_steady_tick(100);
    let mut tasks = Vec::new();
    let semaphore = Arc::new(Semaphore::new(75));
    for downloadable in to_download {
        let permit = semaphore.clone().acquire_owned().await?;
        let progress_bar = progress_bar.clone();
        let output_dir = output_dir.clone();
        tasks.push(spawn(async move {
            let _permit = permit;
            let (size, filename) = downloadable
                .download(
                    &output_dir,
                    |total| {
                        let progress_bar = progress_bar.force_lock();
                        progress_bar.set_length(progress_bar.length() + total);
                    },
                    |additional| {
                        let progress_bar = progress_bar.force_lock();
                        progress_bar.set_position(progress_bar.position() + additional as u64);
                    },
                )
                .await?;
            let progress_bar = progress_bar.force_lock();
            progress_bar.println(format!(
                "{} Downloaded {:7} {}",
                &*TICK,
                match size {
                    Some(size) => size.to_string(size::Base::Base10, size::Style::Smart),
                    None => String::new(),
                },
                filename.dimmed(),
            ));
            Ok::<(), Error>(())
        }));
    }
    for handle in tasks {
        handle.await??;
    }
    Arc::try_unwrap(progress_bar)
        .map_err(|_| anyhow!("Failed to run threads to completion"))?
        .into_inner()?
        .finish_and_clear();
    for installable in to_install {
        if installable.1.is_file() {
            copy(installable.1, output_dir.join(&installable.0)).await?;
        } else if installable.1.is_dir() {
            let mut copy_options = DirCopyOptions::new();
            copy_options.overwrite = true;
            copy_dir(installable.1, &*output_dir, &copy_options)?;
        } else {
            bail!("Could not determine whether installable is file/folder")
        }
        println!(
            "{} Installed          {}",
            &*TICK,
            installable.0.to_string_lossy().dimmed()
        );
    }

    Ok(())
}
