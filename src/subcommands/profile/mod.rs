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
use anyhow::{anyhow, bail, Result};
use colored::Colorize;
use dialoguer::{Confirm, Select};
use ferinth::{structures::tag::GameVersionType, Ferinth};
use fs_extra::dir::{copy, CopyOptions};
use libium::{
    config::structs::{Config, ModLoader},
    file_picker::pick_folder,
    HOME,
};
use std::{fs::read_dir, path::PathBuf};
use tokio::fs::create_dir_all;

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
    let versions = Ferinth::default().list_game_versions().await?;
    let mut major_versions = ["Show all", "Show release"] // Prepend additional options
        .into_iter()
        .chain(
            versions
                .iter()
                .filter(|v| v.major) // Only get major versions
                .map(|v| v.version.as_ref())
                .collect::<Vec<_>>(),
        )
        .collect::<Vec<_>>();
    let selected_version = Select::with_theme(&*THEME)
        .with_prompt("Which version of Minecraft do you play?")
        .items(&major_versions)
        .default(2)
        .interact()?;
    match selected_version {
        0 | 1 => {
            let mut versions = versions
                .into_iter()
                .filter(|v| selected_version == 0 || v.version_type == GameVersionType::Release)
                .map(|v| v.version)
                .collect::<Vec<_>>();
            let selected_version = Select::with_theme(&*THEME)
                .with_prompt("Which version of Minecraft do you play?")
                .items(&versions)
                .interact()?;
            Ok(versions.swap_remove(selected_version))
        }
        _ => Ok(major_versions.swap_remove(selected_version).to_owned()),
    }
}

/// Check that there isn't already a profile with the same name
pub fn check_profile_name(config: &Config, name: &str) -> Result<()> {
    for profile in &config.profiles {
        if profile.name == name {
            bail!("A profile with name {name} already exists");
        }
    }
    Ok(())
}

pub async fn check_output_directory(output_dir: &PathBuf) -> Result<()> {
    if output_dir.is_relative() {
        bail!("The provided output directory is not absolute, i.e. it is a relative path");
    }
    if output_dir.file_name() != Some(std::ffi::OsStr::new("mods")) {
        println!("{}", "Warning! The output directory is not called `mods`. Most mod loaders will load from a directory called `mods`.".bright_yellow());
    }
    let mut backup = false;
    if output_dir.exists() {
        for file in read_dir(output_dir)? {
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
            let backup_dir = pick_folder(
                &HOME,
                "Where should the backup be made?",
                "Output Directory",
            )?
            .ok_or_else(|| anyhow!("Please pick a backup directory"))?;
            create_dir_all(&backup_dir).await?;
            copy(output_dir, backup_dir, &CopyOptions::new())?;
        }
    }
    Ok(())
}
