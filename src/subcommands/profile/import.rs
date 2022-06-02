use anyhow::{anyhow, bail, Result};
use dialoguer::Confirm;
use libium::config::structs::{Config, Profile};
use std::{fs::File, path::PathBuf};

use crate::subcommands::profile::pick_mods_directory;

use super::check_profile_name;

pub async fn import(config: &mut Config, input_path: Option<PathBuf>) -> Result<()> {
    let path = if let Some(path) = input_path {
        path
    } else {
        // TODO make a picker for a file in libium rather than a folder
        bail!("File picker doesn't work yet, specify a path manually");
    };

    let mut profile: Profile = serde_json::de::from_reader(File::open(path)?)?;

    match check_profile_name(config, &profile.name) {
        Ok(_) => {},
        Err(_) => {
            if Confirm::with_theme(&*crate::THEME)
                .with_prompt("Overwrite existing profile?")
                .interact()?
            {
                match config
                    .profiles
                    .iter()
                    .position(|item| item.name == profile.name)
                {
                    Some(index) => {
                        profile.output_dir = config.profiles[index].output_dir.clone();
                        config.profiles[index] = profile;
                        println!("Profile replaced");
                        return Ok(());
                    },
                    None => {
                        return Err(anyhow!("The profile to replace does not exist, somehow..."))
                    },
                }
            } else {
                return Ok(());
            }
        },
    };

    profile.output_dir = pick_mods_directory().await?;

    config.profiles.push(profile);
    config.active_profile = config.profiles.len() - 1; // Make created profile active
    println!("Profile imported");

    Ok(())
}
