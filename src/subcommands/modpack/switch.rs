use crate::THEME;
use anyhow::{anyhow, Result};
use dialoguer::Select;
use libium::config::structs::Config;

pub fn switch(config: &mut Config, modpack_name: Option<String>) -> Result<()> {
    if config.modpacks.len() <= 1 {
        config.active_modpack = 0;
        Err(anyhow!("There is only 1 modpack in your config"))
    } else if let Some(modpack_name) = modpack_name {
        match config
            .modpacks
            .iter()
            .position(|modpack| modpack.name == modpack_name)
        {
            Some(selection) => {
                config.active_modpack = selection;
                Ok(())
            },
            None => Err(anyhow!("The modpack provided does not exist")),
        }
    } else {
        let modpack_names = config
            .modpacks
            .iter()
            .map(|modpack| &modpack.name)
            .collect::<Vec<_>>();

        let selection = Select::with_theme(&*THEME)
            .with_prompt("Select which modpack to switch to")
            .items(&modpack_names)
            .default(config.active_modpack)
            .interact()?;
        config.active_modpack = selection;
        Ok(())
    }
}
