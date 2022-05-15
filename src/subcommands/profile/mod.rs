mod configure;
mod create;
mod delete;
mod list;
mod switch;
pub use configure::configure;
pub use create::create;
pub use delete::delete;
pub use list::list;
pub use switch::switch;

use crate::THEME;
use anyhow::{bail, Result};
use colored::Colorize;
use dialoguer::{Confirm, Select};
use fs_extra::dir::{copy, CopyOptions};
use libium::{
    config::{self, structs::ModLoader},
    file_picker, misc, HOME,
};
use std::path::PathBuf;

pub fn pick_mod_loader(default: Option<&ModLoader>) -> Result<ModLoader> {
    let mut picker = Select::with_theme(&*THEME);
    picker
        .with_prompt("Which mod loader do you use?")
        .items(&["Quilt", "Fabric", "Forge"]);
    if let Some(default) = default {
        picker.default(match default {
            ModLoader::Quilt => 0,
            ModLoader::Fabric => 1,
            ModLoader::Forge => 2,
        });
    }
    match picker.interact()? {
        0 => Ok(ModLoader::Quilt),
        1 => Ok(ModLoader::Fabric),
        2 => Ok(ModLoader::Forge),
        _ => unreachable!(),
    }
}

pub async fn pick_minecraft_version() -> Result<String> {
    let mut latest_versions: Vec<String> = misc::get_major_mc_versions(10).await?;
    let selected_version = Select::with_theme(&*THEME)
        .with_prompt("Which version of Minecraft do you play?")
        .items(&latest_versions)
        .default(0)
        .interact()?;
    Ok(latest_versions.swap_remove(selected_version))
}

/// Check that there isn't already a profile with the same name
pub fn check_profile_name(config: &mut config::structs::Config, name: &str) -> Result<()> {
    for profile in &config.profiles {
        if profile.name == name {
            bail!("A profile with name {} already exists", name);
        }
    }
    Ok(())
}

pub async fn check_output_directory(output_dir: &PathBuf) -> Result<()> {
    if output_dir.is_relative() {
        bail!("The provided output directory is not absolute, i.e. it is a relative path");
    }
    if output_dir.file_name().unwrap() != "mods" {
        println!("{}", "Warning! The output directory is not called `mods`. Most mod loaders will load a directory called `mods`.".bright_yellow());
    }
    let mut backup = false;
    if output_dir.exists() {
        for file in std::fs::read_dir(output_dir)? {
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
