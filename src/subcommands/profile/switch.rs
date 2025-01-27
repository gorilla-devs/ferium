use anyhow::{anyhow, Result};
use colored::Colorize as _;
use inquire::Select;
use libium::{
    config::{filters::ProfileParameters as _, structs::Config},
    iter_ext::IterExt as _,
};

use crate::try_iter_profiles;

pub fn switch(config: &mut Config, profile_name: Option<String>) -> Result<()> {
    if config.profiles.len() <= 1 {
        Err(anyhow!("There is only 1 profile in your config"))
    } else if let Some(profile_name) = profile_name {
        match config.profiles.iter()
            .position(|item| item.name.eq_ignore_ascii_case(&profile_name))
        {
            Some(selection) => {
                config.active_profile = selection;
                Ok(())
            }
            None => Err(anyhow!("The profile provided does not exist")),
        }
    } else {
        let profile_info = try_iter_profiles(&mut config.profiles)
            .map(|(item, profile)| {
                format!(
                    "{:8} {:7} {} {}",
                    profile
                        .filters
                        .mod_loader()
                        .map(|l| l.to_string().purple())
                        .unwrap_or_default(),
                    profile
                        .filters
                        .game_versions()
                        .map(|v| v[0].green())
                        .unwrap_or_default(),
                    item.name.bold(),
                    format!("({} mods)", profile.mods.len()).yellow(),
                )
            })
            .collect_vec();

        let mut select = Select::new("Select which profile to switch to", profile_info);
        if config.active_profile < config.profiles.len() {
            select.starting_cursor = config.active_profile;
        }
        if let Ok(selection) = select.raw_prompt() {
            config.active_profile = selection.index;
        }
        Ok(())
    }
}
