use anyhow::Result;
use ferinth::Ferinth;
use furse::Furse;
use itertools::Itertools;
use octocrab::Octocrab;

pub async fn curseforge(curseforge: &Furse, project_id: i32) -> Result<()> {
    let project = curseforge.get_mod(project_id).await?;
    let authors = project
        .authors
        .iter()
        .map(|author| &author.name)
        .collect::<Vec<_>>();
    let categories = project
        .categories
        .iter()
        .map(|category| &category.name)
        .collect::<Vec<_>>();

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
        authors.iter().format(", "),
        categories.iter().format(", "),
    );

    Ok(())
}

pub async fn modrinth(modrinth: &Ferinth, project_id: &str) -> Result<()> {
    let project = modrinth.get_project(project_id).await?;
    let team_members = modrinth.list_team_members(&project.team).await?;

    // Get the usernames of all the developers
    let developers = team_members
        .iter()
        .map(|member| &member.user.username)
        .collect::<Vec<_>>();

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
        developers.iter().format(", "),
        project.client_side,
        project.server_side,
        project.license.name,
        project
            .license
            .url
            .map_or("".into(), |url| { format!(" ({})", url) }),
    );

    Ok(())
}

/// List all the mods in `profile` with some of their metadata
pub async fn github(github: &Octocrab, full_name: &(String, String)) -> Result<()> {
    let repo_handler = github.repos(&full_name.0, &full_name.1);
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
        repo.html_url.unwrap(),
        downloads,
        repo.owner.unwrap().login,
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

    Ok(())
}
