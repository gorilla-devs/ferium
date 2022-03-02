use crate::error::{Error, Result};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use ferinth::Ferinth;
use furse::Furse;
use libium::config;
use octocrab::Octocrab;

/// Display a list of mods and repos in the profile to select from and remove selected ones
pub async fn remove(
	curseforge: &Furse,
	modrinth: &Ferinth,
	github: &Octocrab,
	profile: &mut config::structs::Profile,
	mod_names: Option<Vec<String>>,
) -> Result<()> {
	let mut names: Vec<String> = Vec::new();

	// Get the names of the mods
	eprint!("Gathering mod names... ");
	for project_id in &profile.modrinth_mods {
		let project = modrinth.get_project(project_id).await?;
		names.push(project.title);
	}

	for repo_name in &profile.github_repos {
		let repo = github.repos(&repo_name.0, &repo_name.1).get().await?;
		names.push(repo.name);
	}

	for project_id in &profile.curse_projects {
		let project = curseforge.get_mod(*project_id).await?;
		names.push(project.name);
	}
	println!("✓");

	let mut items_to_remove = Vec::new();
	match mod_names {
		Some(mod_names) => {
			// Here we use inefficient double nested for loops because
			// 1. We need to retain the indices for removal so we cannot binary search
			// 2. We want to remove duplicates too
			// 3. We want to use the same items_to_remove format so that both options and user selected data can be processed using the same algorithm

			// For each mod name to remove
			for mod_name in mod_names {
				let mut found_mod = false;
				// Search through all the mod names
				for (i, name) in names.iter().enumerate() {
					// If a match is found, push the match's index to items_to_remove
					if name.to_lowercase().contains(&mod_name.to_lowercase()) {
						found_mod = true;
						items_to_remove.push(i);
					}
				}

				// If a mod is not found, throw an error
				if !found_mod {
					return Err(Error::QuitFormatted(format!(
						"A mod called {} is not present in this profile",
						mod_name
					)));
				}
			}
		},
		None => {
			// Show selection menu
			items_to_remove = match MultiSelect::with_theme(&ColorfulTheme::default())
				.with_prompt("Select mods and/or repositories to remove")
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

	// For each mod to remove
	for index in items_to_remove {
		// If index is larger than the length of the repos and modrinth project_ids, then the index is for curse projects
		if index >= (profile.modrinth_mods.len() + profile.github_repos.len()) {
			// Offset the index by the proper amount
			let index = index - (profile.modrinth_mods.len() + profile.github_repos.len());

			// Remove item from profile's curse projects
			profile.curse_projects.swap_remove(index);
		}
		// If index is larger than the mod_ids' length, then the index is for repos
		else if index >= profile.modrinth_mods.len() {
			// Offset the index by the proper amount
			let index = index - profile.modrinth_mods.len();

			// Remove item from profile's repos
			profile.github_repos.swap_remove(index);
		// Or else its for the modrinth project_ids
		} else {
			// Remove item from profile' mod ids
			profile.modrinth_mods.swap_remove(index);
		}
	}

	Ok(())
}
