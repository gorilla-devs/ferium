use crate::error::{Error, Result};
use ferinth::Ferinth;
use furse::Furse;
use libium::config;
use octocrab::Octocrab;

/// List all the mods in `profile` with some of their metadata
pub async fn list(
	curseforge: &Furse,
	modrinth: &Ferinth,
	github: &Octocrab,
	profile: &config::structs::Profile,
) -> Result<()> {
	for project_id in &profile.curse_projects {
		let project = curseforge.get_mod(*project_id).await?;
		let mut authors = String::new();
		for author in project.authors {
			authors.push_str(&author.name);
			authors.push_str(", ");
		}
		// Trim trailing ', '
		authors.truncate(authors.len() - 2);

		let mut categories = String::new();
		for category in project.categories {
			categories.push_str(&category.name);
			categories.push_str(", ");
		}
		// Trim trailing ', '
		categories.truncate(categories.len() - 2);

		println!(
			"{}
			\r  {}\n
			\r  Link:        {}
			\r  Source:      CurseForge Project
			\r  Open Source: {}
			\r  Downloads:   {}
			\r  Authors:     {}
			\r  Category:    {}\n",
			project.name,
			project.summary,
			project.links.website_url,
			project
				.links
				.source_url
				.map_or("No".into(), |url| format!("Yes ({})", url)),
			project.download_count,
			authors,
			categories,
		);
	}

	for project_id in &profile.modrinth_mods {
		// Get project metadata
		let project = modrinth.get_project(project_id).await?;
		let team_members = modrinth.list_team_members(&project.team).await?;

		// Get the usernames of all the developers
		let mut developers = String::new();
		for member in team_members {
			developers.push_str(&member.user.username);
			developers.push_str(", ");
		}
		// Trim trailing ', '
		developers.truncate(developers.len() - 2);

		println!(
			"{}
            \r  {}\n
            \r  Link:           https://modrinth.com/mod/{}
            \r  Source:         Modrinth Mod
            \r  Open Source:    {}
            \r  Downloads:      {}
            \r  Developers:     {}
            \r  Client side:    {:?}
            \r  Server side:    {:?}
            \r  License:        {}{}\n",
			project.title,
			project.description,
			project.slug,
			project
				.source_url
				.map_or("No".into(), |url| { format!("Yes ({})", url) }),
			project.downloads,
			developers,
			project.client_side,
			project.server_side,
			project.license.name,
			project
				.license
				.url
				.map_or("".into(), |url| { format!(" ({})", url) }),
		);
	}

	for repo_name in &profile.github_repos {
		// Get repository metadata
		let repo_handler = github.repos(&repo_name.0, &repo_name.1);
		let repo = repo_handler.get().await?;
		let releases = repo_handler.releases().list().send().await?;
		let mut downloads = 0;

		// Calculate number of downloads
		for release in releases {
			for asset in release.assets {
				downloads += asset.download_count;
			}
		}

		// Print repository data formatted
		println!(
			"{}{}\n
            \r  Link:           {}
            \r  Source:         GitHub Repository
            \r  Downloads:      {}
            \r  Developer:      {}{}\n",
			repo.name,
			repo.description
				.map_or("".into(), |description| { format!("\n  {}", description) }),
			repo.html_url.ok_or(Error::OptionError)?,
			downloads,
			repo.owner.ok_or(Error::OptionError)?.login,
			if let Some(license) = repo.license {
				format!(
					"\n  License:        {}{}",
					license.name,
					license
						.html_url
						.map_or("".into(), |url| { format!(" ({})", url) })
				)
			} else {
				"".into()
			},
		);
	}

	Ok(())
}
