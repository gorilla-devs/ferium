use crate::THEME;

use super::{check_output_directory, check_profile_name, pick_minecraft_version};
use anyhow::{bail, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use libium::{
    config::structs::{Config, ModLoader, Profile},
    file_picker::pick_folder,
    misc::get_minecraft_dir,
};
use std::path::PathBuf;

#[allow(clippy::option_option)]
pub async fn create(
    config: &mut Config,
    import: Option<Option<String>>,
    game_version: Option<String>,
    mod_loader: Option<ModLoader>,
    name: Option<String>,
    output_dir: Option<PathBuf>,
) -> Result<()> {
    let mut profile = match (game_version, mod_loader, name, output_dir) {
        (Some(game_version), Some(mod_loader), Some(name), output_dir) => {
            check_profile_name(config, &name)?;
            let output_dir = output_dir.unwrap_or_else(|| get_minecraft_dir().join("mods"));
            if !output_dir.is_absolute() {
                bail!("The provided output directory is not absolute, i.e. it is a relative path")
            }
            Profile {
                name,
                output_dir,
                game_version,
                mod_loader,
                mods: Vec::new(),
            }
        },
        (None, None, None, None) => {
            let mut selected_mods_dir = get_minecraft_dir().join("mods");
            println!(
                "The default mods directory is {}",
                selected_mods_dir.display()
            );
            if Confirm::with_theme(&*THEME)
                .with_prompt("Would you like to specify a custom mods directory?")
                .interact()?
            {
                if let Some(dir) = pick_folder(&selected_mods_dir, "Pick an output directory").await
                {
                    check_output_directory(&dir).await?;
                    selected_mods_dir = dir;
                };
            }

            let name = loop {
                let name: String = Input::with_theme(&*THEME)
                    .with_prompt("What should this profile be called?")
                    .interact_text()?;

                #[allow(clippy::single_match_else)]
                match check_profile_name(config, &name) {
                    Ok(_) => break name,
                    Err(_) => {
                        println!(
                            "{}",
                            "Please provide a name that is not already being used"
                                .red()
                                .bold()
                        );
                        continue;
                    },
                }
            };

            let selected_version = pick_minecraft_version().await?;

            Profile {
                name,
                output_dir: selected_mods_dir,
                mods: Vec::new(),
                game_version: selected_version,
                mod_loader: super::pick_mod_loader(None)?,
            }
        },
        _ => {
            bail!("Provide at least the name, game version, and mod loader options to create a profile")
        },
    };

    if let Some(from) = import {
        if config.profiles.is_empty() {
            bail!("There are no profiles configured to import mods from")
        }
        // If the profile name has been provided as an option
        let selection = if let Some(profile_name) = from {
            match config
                .profiles
                .iter()
                .position(|profile| profile.name == profile_name)
            {
                Some(selection) => selection,
                None => bail!("The profile name provided does not exist"),
            }
        } else {
            let profile_names = config
                .profiles
                .iter()
                .map(|profile| &profile.name)
                .collect::<Vec<_>>();
            Select::with_theme(&*THEME)
                .with_prompt("Select which profile to import mods from")
                .items(&profile_names)
                .default(config.active_profile)
                .interact()?
        };
        profile.mods = config.profiles[selection].mods.clone();
    }

    println!(
        "{}",
        "After adding your mods, remember to run `ferium upgrade` to download them!".yellow()
    );

    config.profiles.push(profile);
    config.active_profile = config.profiles.len() - 1; // Make created profile active
    Ok(())
}
