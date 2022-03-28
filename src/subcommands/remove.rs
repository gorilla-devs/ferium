use anyhow::{bail, Result};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use libium::config::{self, structs::Mod};

/// Display a list of mods and repos in the profile to select from and remove selected ones
pub fn remove(
	profile: &mut config::structs::Profile,
	mod_names: Option<Vec<String>>,
) -> Result<()> {
	let names: Vec<&str> = profile.mods.iter().map(Mod::name).collect();
	let mut items_to_remove = Vec::new();

	match mod_names {
		Some(mod_names) => {
			for mod_name in mod_names {
				if let Some(index) = names
					.iter()
					.position(|name| name.to_lowercase() == mod_name.to_lowercase())
				{
					items_to_remove.push(index);
				} else {
					bail!("A mod called {} is not present in this profile", mod_name);
				}
			}
		},
		None => {
			items_to_remove = match MultiSelect::with_theme(&ColorfulTheme::default())
				.with_prompt("Select mods to remove")
				.items(&names)
				.interact_opt()?
			{
				Some(items_to_remove) => items_to_remove,
				None => return Ok(()), // Exit if the user cancelled
			};
		},
	}

	// Sort the indices in ascending order to fix moving indices during removal
	items_to_remove.sort_unstable();
	items_to_remove.reverse();
	for index in items_to_remove {
		profile.mods.swap_remove(index);
	}

	Ok(())
}
