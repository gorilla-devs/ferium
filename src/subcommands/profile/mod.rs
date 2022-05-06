mod configure;
mod create;
mod delete;
mod list;
pub use configure::configure;
pub use create::create;
pub use delete::delete;
use fs_extra::dir::{copy, CopyOptions};
pub use list::list;

use crate::THEME;
use anyhow::{bail, Result};
use colored::Colorize;
use dialoguer::{Confirm, Select};
use libium::{
    config::{self, structs::ModLoader},
    file_picker, misc, HOME,
};
use std::path::PathBuf;
use tokio::fs::read_dir;

fn pick_mod_loader(default: Option<&ModLoader>) -> Result<ModLoader> {
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

async fn pick_minecraft_version() -> Result<String> {
    let mut latest_versions: Vec<String> = misc::get_major_mc_versions(10).await?;
    let selected_version = Select::with_theme(&*THEME)
        .with_prompt("Which version of Minecraft do you play?")
        .items(&latest_versions)
        .default(0)
        .interact()?;
    Ok(latest_versions.swap_remove(selected_version))
}

/// Check that there isn't already a profile with the same name
fn check_profile_name(config: &mut config::structs::Config, name: &str) -> Result<()> {
    for profile in &config.profiles {
        if profile.name == name {
            bail!("A profile with name {} already exists", name);
        }
    }
    Ok(())
}

async fn check_output_directory(output_dir: &PathBuf) -> Result<()> {
    if output_dir.is_relative() {
        bail!("The provided output directory is not absolute, i.e. it is a relative path");
    }
    if output_dir.file_name().unwrap() != "mods" {
        println!("{}", "Warning! The output directory is not called `mods`. Most mod loaders will load a directory called `mods`.".bright_yellow());
    }
    if output_dir.exists()
        && read_dir(output_dir).await?.next_entry().await?.is_some()
        && Confirm::with_theme(&*THEME)
            .with_prompt("Your output directory is not empty and it will be erased when upgrading. Would like to create a backup of it?")
            .interact()? {
        let backup_dir = file_picker::pick_folder(&*HOME, "Where should the backup be made?").await.unwrap();
        copy(output_dir, backup_dir, &CopyOptions::new())?;
    }
    Ok(())
}
