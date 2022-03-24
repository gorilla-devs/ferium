use crate::{
	error::{Error, Result},
	subcommands::switch::switch,
};
use dialoguer::{theme::ColorfulTheme, Select};
use libium::config;

pub fn delete(config: &mut config::structs::Config, profile_name: Option<String>) -> Result<()> {
	let selection = match profile_name {
		// If the profile name has been provided as an option
		Some(profile_name) => {
			match config
				.profiles
				.iter()
				.position(|profile| profile.name == profile_name)
			{
				Some(selection) => selection,
				None => return Err(Error::Quit("The profile name provided does not exist")),
			}
		},
		None => {
			let profile_names = config
				.profiles
				.iter()
				.map(|profile| &profile.name)
				.collect::<Vec<_>>();

			let selection = Select::with_theme(&ColorfulTheme::default())
				.with_prompt("Select which profile to delete")
				.items(&profile_names)
				.default(config.active_profile)
				.interact_opt()?;
			if let Some(selection) = selection {
				selection
			} else {
				return Ok(());
			}
		},
	};
	config.profiles.swap_remove(selection);

	// If the currently selected profile is being removed
	if config.active_profile == selection {
		// And there is more than one profile
		if config.profiles.len() > 1 {
			// Let the user pick which profile to switch to
			switch(config, None)?;
		} else {
			config.active_profile = 0;
		}
	}
	Ok(())
}
