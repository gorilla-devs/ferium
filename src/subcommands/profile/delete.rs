use std::cmp::Ordering;

use super::switch;
use crate::THEME;
use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::Select;
use libium::config::structs::Config;

pub fn delete(
    config: &mut Config,
    profile_name: Option<String>,
    switch_to: Option<String>,
) -> Result<()> {
    // If the profile name has been provided as an option
    let selection = if let Some(profile_name) = profile_name {
        config
            .profiles
            .iter()
            .position(|profile| profile.name == profile_name)
            .ok_or_else(|| anyhow!("The profile name provided does not exist"))?
    } else {
        let profile_names = config
            .profiles
            .iter()
            .map(|profile| {
                format!(
                    "{:6} {:7} {} {}",
                    format!("{:?}", profile.mod_loader).purple(),
                    profile.game_version.green(),
                    profile.name.bold(),
                    format!("({} mods)", profile.mods.len()).yellow(),
                )
            })
            .collect::<Vec<_>>();

        let selection = Select::with_theme(&*THEME)
            .with_prompt("Select which profile to delete")
            .items(&profile_names)
            .default(config.active_profile)
            .interact_opt()?;
        if let Some(selection) = selection {
            selection
        } else {
            return Ok(());
        }
    };
    config.profiles.remove(selection);

    match config.active_profile.cmp(&selection) {
        // If the currently selected profile is being removed
        Ordering::Equal => {
            // And there is more than one profile
            if config.profiles.len() > 1 {
                // Let the user pick which profile to switch to
                switch(config, switch_to)?;
            } else {
                config.active_profile = 0;
            }
        }
        // If the active profile comes after the removed profile
        Ordering::Greater => {
            // Decrement the index by one
            config.active_profile -= 1;
        }
        Ordering::Less => (),
    }

    Ok(())
}
