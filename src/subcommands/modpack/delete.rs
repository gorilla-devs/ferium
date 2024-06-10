use std::cmp::Ordering;

use super::switch;
use crate::THEME;
use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::Select;
use libium::config::structs::{Config, ModpackIdentifier};

pub fn delete(
    config: &mut Config,
    modpack_name: Option<String>,
    switch_to: Option<String>,
) -> Result<()> {
    // If the modpack name has been provided as an option
    let selection = if let Some(modpack_name) = modpack_name {
        config
            .modpacks
            .iter()
            .position(|modpack| modpack.name == modpack_name)
            .ok_or_else(|| anyhow!("The modpack name provided does not exist"))?
    } else {
        let modpack_names = config
            .modpacks
            .iter()
            .map(|modpack| {
                format!(
                    "{} {}",
                    match &modpack.identifier {
                        ModpackIdentifier::CurseForgeModpack(id) =>
                            format!("{} {:8}", "CF".red(), id.to_string().dimmed()),
                        ModpackIdentifier::ModrinthModpack(id) =>
                            format!("{} {:8}", "MR".green(), id.dimmed()),
                    },
                    modpack.name.bold(),
                )
            })
            .collect::<Vec<_>>();

        let selection = Select::with_theme(&*THEME)
            .with_prompt("Select which modpack to delete")
            .items(&modpack_names)
            .default(config.active_modpack)
            .interact_opt()?;
        if let Some(selection) = selection {
            selection
        } else {
            return Ok(());
        }
    };
    config.modpacks.remove(selection);

    match config.active_modpack.cmp(&selection) {
        // If the currently selected modpack is being removed
        Ordering::Equal => {
            // And there is more than one modpack
            if config.modpacks.len() > 1 {
                // Let the user pick which modpack to switch to
                switch(config, switch_to)?;
            } else {
                config.active_modpack = 0;
            }
        }
        // If the active modpack comes after the removed modpack
        Ordering::Greater => {
            // Decrement the index by one
            config.active_modpack -= 1;
        }
        Ordering::Less => (),
    }

    Ok(())
}
