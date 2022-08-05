pub mod add;
mod configure;
mod delete;
mod list;
mod switch;
mod upgrade;
pub use configure::configure;
pub use delete::delete;
pub use list::list;
pub use switch::switch;
pub use upgrade::upgrade;

use crate::THEME;
use anyhow::{anyhow, bail, Result};
use dialoguer::Confirm;
use fs_extra::dir::{copy, CopyOptions};
use libium::{file_picker::pick_folder, HOME};
use std::{fs::read_dir, path::Path};

pub async fn check_output_directory(output_dir: &Path) -> Result<()> {
    if output_dir.is_relative() {
        bail!("The provided output directory is not absolute, i.e. it is a relative path");
    }
    for check_dir in [output_dir.join("mods"), output_dir.join("resourcepacks")] {
        let mut backup = false;
        if check_dir.exists() {
            for file in read_dir(&check_dir)? {
                let file = file?;
                if file.path().is_file() && file.file_name() != ".DS_Store" {
                    backup = true;
                    break;
                }
            }
        }
        if backup {
            println!(
                "There are files in the {} folder in your output directory, these will be deleted when you upgrade.",
                check_dir.file_name().expect("Unable to get folder name").to_string_lossy()
            );
            if Confirm::with_theme(&*THEME)
                .with_prompt("Would like to create a backup?")
                .interact()?
            {
                let backup_dir = pick_folder(&*HOME, "Where should the backup be made?")
                    .await
                    .ok_or_else(|| anyhow!("Please pick an output directory"))?;
                copy(check_dir, backup_dir, &CopyOptions::new())?;
            }
        }
    }
    Ok(())
}
