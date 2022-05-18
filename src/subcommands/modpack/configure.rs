use crate::THEME;

use super::check_output_directory;
use anyhow::Result;
use colored::Colorize;
use dialoguer::Confirm;
use libium::{config::structs::Modpack, file_picker};
use std::path::PathBuf;

pub async fn configure(
    modpack: &mut Modpack,
    output_dir: &Option<PathBuf>,
    install_overrides: Option<bool>,
) -> Result<()> {
    match output_dir {
        Some(output_dir) => {
            check_output_directory(output_dir).await?;
            modpack.output_dir = output_dir.clone();
        },
        None => {
            if let Some(dir) =
                file_picker::pick_folder(&modpack.output_dir, "Pick an output directory").await
            {
                check_output_directory(&dir).await?;
                modpack.output_dir = dir;
            }
        },
    }
    match install_overrides {
        Some(install_overrides) => modpack.install_overrides = install_overrides,
        None => {
            let install_overrides = Confirm::with_theme(&*THEME)
                .default(true)
                .with_prompt("Should overrides be installed?")
                .interact()?;
            if install_overrides {
                println!(
                    "{}",
                    "WARNING: Configs in your output directory may be overwritten by modpack overrides"
                        .yellow()
                );
            }
            modpack.install_overrides = install_overrides;
        },
    }
    Ok(())
}
