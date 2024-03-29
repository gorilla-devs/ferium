use crate::THEME;
use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::Select;
use libium::config::structs::{Config, ModpackIdentifier};

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
            }
            None => Err(anyhow!("The modpack provided does not exist")),
        }
    } else {
        let modpack_info = config
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
            .with_prompt("Select which modpack to switch to")
            .items(&modpack_info)
            .default(config.active_modpack)
            .interact()?;
        config.active_modpack = selection;
        Ok(())
    }
}
