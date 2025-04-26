use crate::TICK;
use anyhow::{Context as _, Result};
use colored::Colorize as _;
use ferinth::structures::{project::Project, user::TeamMember};
use furse::structures::mod_structs::Mod;
use libium::{
    config::structs::{ModIdentifier, Profile},
    iter_ext::IterExt as _,
    CURSEFORGE_API, GITHUB_API, MODRINTH_API,
};
use octocrab::models::{repos::Release, Repository};
use tokio::task::JoinSet;

enum Metadata {
    CF(Box<Mod>),
    MD(Box<Project>, Vec<TeamMember>),
    GH(Box<Repository>, Vec<Release>),
}
impl Metadata {
    fn name(&self) -> &str {
        match self {
            Metadata::CF(p) => &p.name,
            Metadata::MD(p, _) => &p.title,
            Metadata::GH(p, _) => &p.name,
        }
    }

    #[expect(clippy::unwrap_used)]
    fn id(&self) -> ModIdentifier {
        match self {
            Metadata::CF(p) => ModIdentifier::CurseForgeProject(p.id),
            Metadata::MD(p, _) => ModIdentifier::ModrinthProject(p.id.clone()),
            Metadata::GH(p, _) => {
                ModIdentifier::GitHubRepository(p.owner.clone().unwrap().login, p.name.clone())
            }
        }
    }

    fn slug(&self) -> &str {
        match self {
            Metadata::CF(p) => &p.slug,
            Metadata::MD(p, _) => &p.slug,
            Metadata::GH(p, _) => &p.name,
        }
    }
}

pub async fn verbose(profile: &mut Profile, markdown: bool) -> Result<()> {
    if !markdown {
        eprint!("Querying metadata... ");
    }

    let mut tasks = JoinSet::new();
    let mut mr_ids = Vec::new();
    let mut cf_ids = Vec::new();
    for mod_ in &profile.mods {
        match mod_.identifier.clone() {
            ModIdentifier::CurseForgeProject(project_id) => cf_ids.push(project_id),
            ModIdentifier::ModrinthProject(project_id) => mr_ids.push(project_id),
            ModIdentifier::GitHubRepository(owner, repo) => {
                let repo = GITHUB_API.repos(owner, repo);
                tasks.spawn(async move {
                    Ok::<_, anyhow::Error>((
                        repo.get().await?,
                        repo.releases().list().send().await?,
                    ))
                });
            }
            _ => todo!(),
        }
    }

    let mr_projects = if mr_ids.is_empty() {
        vec![]
    } else {
        MODRINTH_API
            .project_get_multiple(&mr_ids.iter().map(AsRef::as_ref).collect_vec())
            .await?
    };
    let mr_teams_members = if mr_projects.is_empty() {
        vec![]
    } else {
        MODRINTH_API
            .team_multiple_list_members(&mr_projects.iter().map(|p| p.team.as_ref()).collect_vec())
            .await?
    };

    let cf_projects = if cf_ids.is_empty() {
        vec![]
    } else {
        CURSEFORGE_API.get_mods(cf_ids).await?
    };

    let mut metadata = Vec::new();
    for (project, members) in mr_projects.into_iter().zip(mr_teams_members) {
        metadata.push(Metadata::MD(Box::new(project), members));
    }
    for project in cf_projects {
        metadata.push(Metadata::CF(Box::new(project)));
    }
    for res in tasks.join_all().await {
        let (repo, releases) = res?;
        metadata.push(Metadata::GH(Box::new(repo), releases.items));
    }
    metadata.sort_unstable_by_key(|e| e.name().to_lowercase());

    if !markdown {
        println!("{}", &*TICK);
    }

    for project in &metadata {
        let mod_ = profile
            .mods
            .iter_mut()
            .find(|mod_| mod_.identifier == project.id())
            .context("Could not find expected mod")?;

        mod_.name = project.name().to_string();
        mod_.slug = Some(project.slug().to_string());

        if markdown {
            match project {
                Metadata::CF(p) => curseforge_md(p),
                Metadata::MD(p, t) => modrinth_md(p, t),
                Metadata::GH(p, _) => github_md(p),
            }
        } else {
            match project {
                Metadata::CF(p) => curseforge(p),
                Metadata::MD(p, t) => modrinth(p, t),
                Metadata::GH(p, r) => github(p, r),
            }
        }
    }

    Ok(())
}

pub fn curseforge(project: &Mod) {
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
        project.links.website_url.to_string().blue().underline(),
        "CurseForge Mod".dimmed(),
        project.id.to_string().dimmed(),
        project
            .links
            .source_url
            .as_ref()
            .map_or("No".red(), |url| format!(
                "Yes ({})",
                url.to_string().blue().underline()
            )
            .green()),
        project.download_count.to_string().yellow(),
        project
            .authors
            .iter()
            .map(|author| &author.name)
            .display(", ")
            .to_string()
            .cyan(),
        project
            .categories
            .iter()
            .map(|category| &category.name)
            .display(", ")
            .to_string()
            .magenta(),
    );
}

pub fn modrinth(project: &Project, team_members: &[TeamMember]) {
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
        project.source_url.as_ref().map_or("No".red(), |url| {
            format!("Yes ({})", url.to_string().blue().underline()).green()
        }),
        project.downloads.to_string().yellow(),
        team_members
            .iter()
            .map(|member| &member.user.username)
            .display(", ")
            .to_string()
            .cyan(),
        project
            .categories
            .iter()
            .display(", ")
            .to_string()
            .magenta(),
        {
            if project.license.name.is_empty() {
                "Custom"
            } else {
                &project.license.name
            }
        },
        project.license.url.as_ref().map_or(String::new(), |url| {
            format!(" ({})", url.to_string().blue().underline())
        }),
    );
}

#[expect(clippy::unwrap_used)]
pub fn github(repo: &Repository, releases: &[Release]) {
    // Calculate number of downloads
    let mut downloads = 0;
    for release in releases {
        for asset in &release.assets {
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
        &repo.name.bold(),
        repo.description
            .as_ref()
            .map_or(String::new(), |description| {
                format!("\n  {description}")
            })
            .italic(),
        repo.html_url
            .as_ref()
            .unwrap()
            .to_string()
            .blue()
            .underline(),
        "GitHub Repository".dimmed(),
        repo.full_name.as_ref().unwrap().dimmed(),
        "Yes".green(),
        downloads.to_string().yellow(),
        repo.owner.as_ref().unwrap().login.cyan(),
        repo.topics.as_ref().map_or("".into(), |topics| topics
            .iter()
            .display(", ")
            .to_string()
            .magenta()),
        repo.license
            .as_ref()
            .map_or("None".into(), |license| format!(
                "{}{}",
                license.name,
                license.html_url.as_ref().map_or(String::new(), |url| {
                    format!(" ({})", url.to_string().blue().underline())
                })
            )),
    );
}

pub fn curseforge_md(project: &Mod) {
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
            .as_ref()
            .map_or("No".into(), |url| format!("[Yes]({url})")),
        project
            .authors
            .iter()
            .map(|author| format!("[{}]({})", author.name, author.url))
            .display(", "),
        project
            .categories
            .iter()
            .map(|category| &category.name)
            .display(", "),
    );
}

pub fn modrinth_md(project: &Project, team_members: &[TeamMember]) {
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
            .as_ref()
            .map_or("No".into(), |url| { format!("[Yes]({url})") }),
        team_members
            .iter()
            .map(|member| format!(
                "[{}](https://modrinth.com/user/{})",
                member.user.username, member.user.id
            ))
            .display(", "),
        project.categories.iter().display(", "),
    );
}

#[expect(clippy::unwrap_used)]
pub fn github_md(repo: &Repository) {
    println!(
        "
**[{}]({})**{}

|             |             |
|-------------|-------------|
| Source      | GitHub `{}` |
| Open Source | Yes         |
| Owner       | [{}]({})    |{}",
        repo.name,
        repo.html_url.as_ref().unwrap(),
        repo.description
            .as_ref()
            .map_or(String::new(), |description| {
                format!("  \n_{}_", description.trim())
            }),
        repo.full_name.as_ref().unwrap(),
        repo.owner.as_ref().unwrap().login,
        repo.owner.as_ref().unwrap().html_url,
        repo.topics.as_ref().map_or(String::new(), |topics| format!(
            "\n| Topics | {} |",
            topics.iter().display(", ")
        )),
    );
}
