use anyhow::{anyhow, Result};
use colored::Colorize as _;
use inquire::Select;
use libium::{
    config::structs::{Config, ModpackIdentifier},
    iter_ext::IterExt as _,
};

pub fn switch(config: &mut Config, modpack_name: Option<String>) -> Result<()> {
    if config.modpacks.len() <= 1 {
        config.active_modpack = 0;
        Err(anyhow!("There is only 1 modpack in your config"))
    } else if let Some(modpack_name) = modpack_name {
        match config
            .modpacks
            .iter()
            .position(|modpack| modpack.name.eq_ignore_ascii_case(&modpack_name))
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
            .collect_vec();

        let mut select = Select::new("Select which modpack to switch to", modpack_info);
        if config.active_modpack < config.modpacks.len() {
            select.starting_cursor = config.active_modpack;
        }
        if let Ok(selection) = select.raw_prompt() {
            config.active_modpack = selection.index;
        }
        Ok(())
    }
}
