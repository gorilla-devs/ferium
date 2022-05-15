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
use anyhow::{bail, Result};
use dialoguer::Confirm;
use fs_extra::dir::{copy, CopyOptions};
use libium::{file_picker, HOME};
use std::path::PathBuf;

pub async fn check_output_directory(output_dir: &PathBuf) -> Result<()> {
    if output_dir.is_relative() {
        bail!("The provided output directory is not absolute, i.e. it is a relative path");
    }
    let mut backup = false;
    if output_dir.exists() {
        for file in std::fs::read_dir(output_dir.join("mods"))? {
            let file = file?;
            if file.path().is_file() && file.file_name() != ".DS_Store" {
                backup = true;
                break;
            }
        }
    }
    if backup {
        println!(
            "There are files in your output directory, these will be deleted when you upgrade."
        );
        if Confirm::with_theme(&*THEME)
            .with_prompt("Would like to create a backup?")
            .interact()?
        {
            let backup_dir = file_picker::pick_folder(&*HOME, "Where should the backup be made?")
                .await
                .unwrap();
            copy(output_dir, backup_dir, &CopyOptions::new())?;
        }
    }
    Ok(())
}
