use super::{check_profile_name, pick_minecraft_version};
use crate::subcommands::profile::pick_mods_directory;
use anyhow::{bail, Result};
use dialoguer::{Input, Select};
use libium::config;
use std::path::PathBuf;

#[allow(clippy::option_option)]
pub async fn create(
    config: &mut config::structs::Config,
    import: Option<Option<String>>,
    game_version: Option<String>,
    mod_loader: Option<config::structs::ModLoader>,
    name: Option<String>,
    output_dir: Option<PathBuf>,
) -> Result<()> {
    let mut profile = match (game_version, mod_loader, name, output_dir) {
        (Some(game_version), Some(mod_loader), Some(name), Some(output_dir)) => {
            check_profile_name(config, &name)?;
            if !output_dir.is_absolute() {
                bail!("The provided output directory is not absolute, i.e. it is a relative path")
            }
            config::structs::Profile {
                name,
                output_dir,
                game_version,
                mod_loader,
                mods: Vec::new(),
            }
        },
        (None, None, None, None) => {
            // Create profile using a UI
            println!("Please enter the details for the new profile");

            // Let user pick mods directory
            let selected_mods_dir = pick_mods_directory().await?;

            let name = loop {
                let name: String = Input::with_theme(&*crate::THEME)
                    .with_prompt("What should this profile be called?")
                    .interact_text()?;

                match check_profile_name(config, &name) {
                    Ok(_) => break name,
                    Err(_) => continue,
                }
            };

            let selected_version = pick_minecraft_version().await?;

            config::structs::Profile {
                name,
                output_dir: selected_mods_dir,
                mods: Vec::new(),
                game_version: selected_version,
                mod_loader: super::pick_mod_loader(None)?,
            }
        },
        _ => {
            bail!("Provide all four arguments to create a profile using options")
        },
    };

    if let Some(from) = import {
        if config.profiles.is_empty() {
            bail!("There are no profiles configured to import mods from")
        }
        let selection = match from {
            // If the profile name has been provided as an option
            Some(profile_name) => {
                match config
                    .profiles
                    .iter()
                    .position(|profile| profile.name == profile_name)
                {
                    Some(selection) => selection,
                    None => bail!("The profile name provided does not exist"),
                }
            },
            None => {
                let profile_names = config
                    .profiles
                    .iter()
                    .map(|profile| &profile.name)
                    .collect::<Vec<_>>();
                Select::with_theme(&*crate::THEME)
                    .with_prompt("Select which profile to import mods from")
                    .items(&profile_names)
                    .default(config.active_profile)
                    .interact()?
            },
        };
        profile.mods = config.profiles[selection].mods.clone();
    }

    config.profiles.push(profile);
    config.active_profile = config.profiles.len() - 1; // Make created profile active
    Ok(())
}
