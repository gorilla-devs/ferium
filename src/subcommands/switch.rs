use crate::error::{Error, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use libium::config;

pub fn switch(config: &mut config::structs::Config, profile_name: Option<String>) -> Result<()> {
	if config.profiles.len() < 2 {
		Err(Error::Quit("There is less than 2 profiles in your config"))
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
			None => Err(Error::Quit("The profile provided does not exist")),
		}
	} else {
		let profile_names = config
			.profiles
			.iter()
			.map(|profile| &profile.name)
			.collect::<Vec<_>>();

		let selection = Select::with_theme(&ColorfulTheme::default())
			.with_prompt("Select which profile to switch to")
			.items(&profile_names)
			.default(config.active_profile)
			.interact()?;
		config.active_profile = selection;
		Ok(())
	}
}
