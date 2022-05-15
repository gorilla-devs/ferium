use crate::{STYLE, TICK};
use anyhow::{Error, Result};
use colored::Colorize;
use indicatif::ProgressBar;
use libium::{mutex_ext::MutexExt, upgrade::Downloadable};
use std::{
    ffi::OsString,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{fs::copy, spawn};

pub async fn download(
    output_dir: Arc<PathBuf>,
    to_download: Vec<Downloadable>,
    to_install: Vec<(OsString, PathBuf)>,
) -> Result<()> {
    if to_download.is_empty() && to_install.is_empty() {
        println!("\n{}", "All up to date!".bold());
    } else {
        println!("\n{}\n", "Downloading Mod Files".bold());
        let progress_bar = Arc::new(Mutex::new(
            ProgressBar::new(to_download.len() as u64).with_style(STYLE.clone()),
        ));
        {
            progress_bar.force_lock().enable_steady_tick(100);
        }
        let mut tasks = Vec::new();
        for downloadable in to_download {
            let progress_bar = progress_bar.clone();
            let output_dir = output_dir.clone();
            tasks.push(spawn(async move {
                let (size, filename) = downloadable.download(&output_dir, |_, _| ()).await?;
                let progress_bar = progress_bar.force_lock();
                progress_bar.println(format!(
                    "{} Downloaded {:7} {}",
                    &*TICK,
                    size.to_string(size::Base::Base10, size::Style::Smart),
                    filename.dimmed(),
                ));
                progress_bar.set_position(progress_bar.position() + 1);
                Ok::<(), Error>(())
            }));
        }
        for handle in tasks {
            handle.await??;
        }
        Arc::try_unwrap(progress_bar)
            .expect("Failed to run threads to completion")
            .into_inner()?
            .finish_and_clear();
        for installable in to_install {
            eprint!(
                "Installing  {}... ",
                installable.0.to_string_lossy().dimmed()
            );
            copy(installable.1, output_dir.join(installable.0)).await?;
            println!("{}", &*TICK);
        }
    }

    Ok(())
}
