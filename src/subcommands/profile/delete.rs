use super::switch;
use crate::THEME;
use anyhow::{bail, Result};
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
    config.profiles.swap_remove(selection);

    // If the currently selected profile is being removed
    if config.active_profile == selection {
        // And there is more than one profile
        if config.profiles.len() > 1 {
            // Let the user pick which profile to switch to
            switch(config, switch_to)?;
        } else {
            config.active_profile = 0;
        }
    }
    Ok(())
}
