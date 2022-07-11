use anyhow::Result;
use colored::Colorize;
use ferinth::Ferinth;
use furse::Furse;
use itertools::Itertools;
use octocrab::Octocrab;
use std::sync::Arc;

pub async fn curseforge(curseforge: Arc<Furse>, project_id: i32) -> Result<()> {
    let project = curseforge.get_mod(project_id).await?;
    println!(
        "
{}
  {}\n
  Link:         {}
  Source:       {}
  Project ID:   {}
  Open Source:  {}
  Downloads:    {}
  Authors:      {}
  Categories:   {}",
        project.name.bold(),
        project.summary.trim().italic(),
        project.links.website_url.blue().underline(),
        "CurseForge Mod".dimmed(),
        project.id.to_string().dimmed(),
        project.links.source_url.map_or("No".red(), |url| format!(
            "Yes ({})",
            url.blue().underline()
        )
        .green()),
        project.download_count.to_string().yellow(),
        project
            .authors
            .iter()
            .map(|author| &author.name)
            .format(", ")
            .to_string()
            .cyan(),
        project
            .categories
            .iter()
            .map(|category| &category.name)
            .format(", ")
            .to_string()
            .magenta(),
    );

    Ok(())
}

pub async fn modrinth(modrinth: Arc<Ferinth>, project_id: String) -> Result<()> {
    let project = modrinth.get_project(&project_id).await?;
    let team_members = modrinth.list_team_members(&project.team).await?;

    println!(
        "
{}
  {}\n
  Link:         {}
  Source:       {}
  Project ID:   {}
  Open Source:  {}
  Downloads:    {}
  Authors:      {}
  Categories:   {}
  License:      {}{}",
        project.title.bold(),
        project.description.italic(),
        format!("https://modrinth.com/mod/{}", project.slug)
            .blue()
            .underline(),
        "Modrinth Mod".dimmed(),
        project.id.dimmed(),
        project.source_url.map_or("No".red(), |url| {
            format!("Yes ({})", url.blue().underline()).green()
        }),
        project.downloads.to_string().yellow(),
        team_members
            .iter()
            .map(|member| &member.user.username)
            .format(", ")
            .to_string()
            .cyan(),
        project.categories.iter().format(", ").to_string().magenta(),
        project.license.name,
        project.license.url.map_or("".into(), |url| {
            format!(" ({})", url.blue().underline())
        }),
    );

    Ok(())
}

pub async fn github(github: Arc<Octocrab>, full_name: (String, String)) -> Result<()> {
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

    println!(
        "
{}{}\n
  Link:         {}
  Source:       {}
  Identifier:   {}
  Open Source:  {}
  Downloads:    {}
  Authors:      {}
  Topics:       {}
  License:      {}",
        repo.name.bold(),
        repo.description
            .map_or("".into(), |description| { format!("\n  {}", description) })
            .italic(),
        repo.html_url.unwrap().to_string().blue().underline(),
        "GitHub Repository".dimmed(),
        repo.full_name.unwrap().dimmed(),
        "Yes".green(),
        downloads.to_string().yellow(),
        repo.owner.unwrap().login.cyan(),
        repo.topics.map_or("".into(), |topics| topics
            .iter()
            .format(", ")
            .to_string()
            .magenta()),
        repo.license.map_or("None".into(), |license| format!(
            "{}{}",
            license.name,
            license.html_url.map_or("".into(), |url| {
                format!(" ({})", url.to_string().blue().underline())
            })
        )),
    );

    Ok(())
}

pub async fn curseforge_md(curseforge: Arc<Furse>, project_id: i32) -> Result<()> {
    let project = curseforge.get_mod(project_id).await?;
    println!(
        "
**[{}]({})**  
_{}_

|             |                 |
|-------------|-----------------|
| Source      | CurseForge `{}` |
| Open Source | {}              |
| Authors     | {}              |
| Categories  | {}              |",
        project.name.trim(),
        project.links.website_url,
        project.summary.trim(),
        project.id,
        project
            .links
            .source_url
            .map_or("No".into(), |url| format!("[Yes]({})", url)),
        project
            .authors
            .iter()
            .map(|author| format!("[{}]({})", author.name, author.url))
            .format(", "),
        project
            .categories
            .iter()
            .map(|category| &category.name)
            .format(", "),
    );

    Ok(())
}

pub async fn modrinth_md(modrinth: Arc<Ferinth>, project_id: String) -> Result<()> {
    let project = modrinth.get_project(&project_id).await?;
    let team_members = modrinth.list_team_members(&project.team).await?;

    println!(
        "
**[{}](https://modrinth.com/mod/{})**  
_{}_

|             |               |
|-------------|---------------|
| Source      | Modrinth `{}` |
| Open Source | {}            |
| Author      | {}            |
| Categories  | {}            |",
        project.title.trim(),
        project.id,
        project.description.trim(),
        project.id,
        project
            .source_url
            .map_or("No".into(), |url| { format!("[Yes]({})", url) }),
        team_members
            .iter()
            .map(|member| format!(
                "[{}](https://modrinth.com/user/{})",
                member.user.username, member.user.id
            ))
            .format(", "),
        project.categories.iter().format(", "),
    );

    Ok(())
}

pub async fn github_md(github: Arc<Octocrab>, full_name: (String, String)) -> Result<()> {
    let repo_handler = github.repos(&full_name.0, &full_name.1);
    let repo = repo_handler.get().await?;

    println!(
        "
**[{}]({})**{}

|             |             |
|-------------|-------------|
| Source      | GitHub `{}` |
| Open Source | Yes         |
| Owner       | [{}]({})    |{}",
        repo.name,
        repo.html_url.unwrap(),
        repo.description.map_or("".into(), |description| {
            format!("  \n_{}_", description.trim())
        }),
        repo.full_name.unwrap(),
        repo.owner.clone().unwrap().login,
        repo.owner.unwrap().html_url,
        repo.topics.map_or("".into(), |topics| format!(
            "\n| Topics | {} |",
            topics.iter().format(", ")
        )),
    );

    Ok(())
}
