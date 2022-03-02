use crate::error::{Error, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use libium::config;

pub fn delete(config: &mut config::structs::Config, profile_name: Option<String>) -> Result<()> {
	let selection = match profile_name {
		// If the profile name has been provided as an option
		Some(profile_name) => {
			// Sort profiles by their names
			config
				.profiles
				.sort_unstable_by_key(|profile| profile.name.clone());
			// Binary search the profile by their names
			match config
				.profiles
				.binary_search_by_key(&&profile_name, |profile| &profile.name)
			{
				// If the profile is found, return its index
				Ok(selection) => selection,
				// Else return an error
				Err(_) => return Err(Error::Quit("The profile name provided does not exist")),
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
			// Remove provided profile if one was selected
			if let Some(selection) = selection {
				selection
			} else {
				return Ok(());
			}
		},
	};
	// If the currently selected profile is being removed
	if config.active_profile == selection {
		// Default to the first profile
		config.active_profile = 0;
	}
	// Remove provided profile
	config.profiles.swap_remove(selection);
	Ok(())
}
