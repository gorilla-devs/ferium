use crate::THEME;
use anyhow::{anyhow, Result};
use dialoguer::Select;
use libium::config::structs::Config;

pub fn switch(config: &mut Config, profile_name: Option<String>) -> Result<()> {
    if config.profiles.len() <= 1 {
        Err(anyhow!("There is only 1 profile in your config"))
    } else if let Some(profile_name) = profile_name {
        match config
            .profiles
            .iter()
            .position(|profile| profile.name == profile_name)
        {
            Some(selection) => {
                config.active_profile = selection;
                Ok(())
            },
            None => Err(anyhow!("The profile provided does not exist")),
        }
    } else {
        let profile_names = config
            .profiles
            .iter()
            .map(|profile| &profile.name)
            .collect::<Vec<_>>();

        let selection = Select::with_theme(&*THEME)
            .with_prompt("Select which profile to switch to")
            .items(&profile_names)
            .default(config.active_profile)
            .interact()?;
        config.active_profile = selection;
        Ok(())
    }
}
