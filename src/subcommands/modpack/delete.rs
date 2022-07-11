use super::switch;
use crate::THEME;
use anyhow::{bail, Result};
use dialoguer::Select;
use libium::config::structs::Config;

pub fn delete(config: &mut Config, modpack_name: Option<String>) -> Result<()> {
    let selection = match modpack_name {
        // If the modpack name has been provided as an option
        Some(modpack_name) => {
            match config
                .modpacks
                .iter()
                .position(|modpack| modpack.name == modpack_name)
            {
                Some(selection) => selection,
                None => bail!("The modpack name provided does not exist"),
            }
        },
        None => {
            let modpack_names = config
                .modpacks
                .iter()
                .map(|modpack| &modpack.name)
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
        },
    };
    config.modpacks.swap_remove(selection);

    // If the currently selected modpack is being removed
    if config.active_modpack == selection {
        // And there is more than one modpack
        if config.modpacks.len() > 1 {
            // Let the user pick which modpack to switch to
            switch(config, None)?;
        } else {
            config.active_modpack = 0;
        }
    }
    Ok(())
}
