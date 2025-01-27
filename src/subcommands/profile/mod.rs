mod configure;
mod create;
mod delete;
mod info;
mod switch;
mod import;
pub use configure::configure;
pub use create::create;
pub use delete::delete;
pub use info::info;
pub use switch::switch;
pub use import::import;

use crate::file_picker::pick_folder;
use anyhow::{ensure, Context as _, Result};
use colored::{ColoredString, Colorize as _};
use ferinth::Ferinth;
use fs_extra::dir::{copy, CopyOptions};
use inquire::{list_option::ListOption, validator::{ErrorMessage, Validation}, Confirm, MultiSelect, Select};
use libium::{config::structs::ModLoader, iter_ext::IterExt as _, HOME};
use std::{
    fs::{create_dir_all, read_dir},
    path::PathBuf,
};

#[expect(clippy::unwrap_used, reason = "All variants are present")]
pub fn pick_mod_loader(default: Option<&ModLoader>) -> Result<ModLoader> {
    let options = vec![
        ModLoader::Fabric,
        ModLoader::Quilt,
        ModLoader::NeoForge,
        ModLoader::Forge,
    ];
    let mut picker = Select::new("Which mod loader do you use?", options.clone())
        .without_filtering()
        .without_help_message();
    if let Some(default) = default {
        picker.starting_cursor = options.iter().position(|l| l == default).unwrap();
    }
    Ok(picker.prompt()?)
}

pub async fn pick_minecraft_versions(default: &[String]) -> Result<Vec<String>> {
    let mut versions = Ferinth::default().list_game_versions().await?;
    versions.sort_by(|a, b| {
        // Sort by release type (release > snapshot > beta > alpha) then in reverse chronological order
        a.version_type
            .cmp(&b.version_type)
            .then(b.date.cmp(&a.date))
    });
    let mut default_indices = vec![];
    let display_versions = versions
        .iter()
        .enumerate()
        .map(|(i, v)| {
            if default.contains(&v.version) {
                default_indices.push(i);
            }
            if v.major {
                v.version.bold()
            } else {
                v.version.clone().into()
            }
        })
        .collect_vec();

    let selected_versions =
        MultiSelect::new("Which version of Minecraft do you play?", display_versions)
            .with_validator(|x: &[ListOption<&ColoredString>]| if x.is_empty() {
                Ok(Validation::Invalid(ErrorMessage::Custom("You need to select atleast one version".to_owned())))
            } else {
                Ok(Validation::Valid)
            })
            .with_default(&default_indices)
            .raw_prompt()?
            .into_iter()
            .map(|s| s.index)
            .collect_vec();

    Ok(versions
        .into_iter()
        .enumerate()
        .filter_map(|(i, v)| {
            if selected_versions.contains(&i) {
                Some(v.version)
            } else {
                None
            }
        })
        .collect_vec())
}

pub async fn check_output_directory(output_dir: &PathBuf, no_gui_mode: Option<bool>) -> Result<()> {
    ensure!(
        output_dir.is_absolute(),
        "The provided output directory is not absolute, i.e. it is a relative path"
    );
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
        if Confirm::new("Would like to create a backup?")
            .prompt()
            .unwrap_or_default()
        {
            let backup_dir = pick_folder(
                &*HOME,
                "Where should the backup be made?",
                "Output Directory",
                no_gui_mode,
            )?
            .context("Please pick a backup directory")?;
            create_dir_all(&backup_dir)?;
            copy(output_dir, backup_dir, &CopyOptions::new())?;
        }
    }
    Ok(())
}
