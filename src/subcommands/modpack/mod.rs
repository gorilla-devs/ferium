pub mod add;
mod configure;
mod delete;
mod info;
mod switch;
mod upgrade;
pub use configure::configure;
pub use delete::delete;
pub use info::info;
pub use switch::switch;
pub use upgrade::upgrade;

use crate::file_picker::pick_file;
use anyhow::{ensure, Context as _, Result};
use fs_extra::dir::{copy, CopyOptions};
use inquire::Confirm;
use libium::HOME;
use std::{fs::read_dir, path::Path};

pub fn check_output_directory(output_dir: &Path, no_gui_mode: Option<bool>) -> Result<()> {
    ensure!(
        output_dir.is_absolute(),
        "The provided output directory is not absolute, i.e. it is a relative path"
    );

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
                check_dir.file_name().context("Unable to get folder name")?.to_string_lossy()
            );
            if Confirm::new("Would like to create a backup?")
                .prompt()
                .unwrap_or_default()
            {
                let backup_dir = pick_file(
                    &*HOME,
                    "Where should the backup be made?",
                    "Output Directory",
                    true,
                    no_gui_mode,
                )?
                .context("Please pick an output directory")?;
                copy(check_dir, backup_dir, &CopyOptions::new())?;
            }
        }
    }
    Ok(())
}
