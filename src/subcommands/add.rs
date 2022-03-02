use crate::error::{Error, Result};
use ferinth::Ferinth;
use furse::Furse;
use libium::config;
use octocrab::Octocrab;

/// Check if repo `owner`/`repo_name` exists and releases mods, and if so add repo to `profile`
pub async fn github(
	github: &Octocrab,
	owner: String,
	repo_name: String,
	profile: &mut config::structs::Profile,
) -> Result<()> {
	eprint!("Adding GitHub repository... ");

	// Get repository and releases data
	let repo_handler = github.repos(owner, repo_name);
	let repo = repo_handler.get().await?;
	// Get the name of the repository as a tuple
	let repo_name_split = repo
		.full_name
		.as_ref()
		.ok_or(Error::OptionError)?
		.split('/')
		.collect::<Vec<_>>();
	let repo_name = (repo_name_split[0].into(), repo_name_split[1].into());

	// Check if repo has already been added
	if profile.github_repos.contains(&repo_name) {
		return Err(Error::Quit("× Repository already added to profile!"));
	}

	let releases = repo_handler.releases().list().send().await?;
	let mut contains_jar_asset = false;

	// Search every asset to check if the releases contain JAR files (a mod file)
	'outer: for release in releases {
		for asset in release.assets {
			if asset.name.contains("jar") {
				// If JAR release is found, set flag to true and break
				contains_jar_asset = true;
				break 'outer;
			}
		}
	}

	if contains_jar_asset {
		// Append repo to profile
		profile.github_repos.push(repo_name);
		println!("✓");
	} else {
		return Err(Error::Quit("× Repository does not release mods!"));
	}

	Ok(())
}

/// Check if mod with ID `mod_id` exists, if so add that mod to `profile`
pub async fn modrinth(
	modrinth: &Ferinth,
	project_id: String,
	profile: &mut config::structs::Profile,
) -> Result<()> {
	eprint!("Adding Modrinth mod... ");

	// Check if mod exists
	match modrinth.get_project(&project_id).await {
		Ok(project) => {
			// Check if mod has already been added
			if profile.modrinth_mods.contains(&project.id) {
				return Err(Error::Quit("× Mod already added to profile!"));
			}
			// And if it hasn't, append mod to profile and write
			profile.modrinth_mods.push(project.id);
			println!("✓ ({})", project.title);

			Ok(())
		},
		Err(_) => {
			// Else return an error
			Err(Error::QuitFormatted(format!(
				"× Mod with ID `{}` does not exist!",
				project_id
			)))
		},
	}
}

pub async fn curseforge(
	curseforge: &Furse,
	project_id: i32,
	profile: &mut config::structs::Profile,
) -> Result<()> {
	eprint!("Adding CurseForge mod... ");

	// Check if project exists
	match curseforge.get_mod(project_id).await {
		Ok(project) => {
			if profile.curse_projects.contains(&project.id) {
				Err(Error::Quit("× Project already added to profile!"))
			} else {
				profile.curse_projects.push(project.id);
				println!("✓ ({})", project.name);
				Ok(())
			}
		},
		Err(err) => Err(Error::QuitFormatted(format!(
			"× Project with ID `{}` does not exist! ({})",
			project_id, err
		))),
	}
}
