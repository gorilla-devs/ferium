use crate::{THEME, TICK};
use anyhow::{anyhow, bail, Result};
use dialoguer::Confirm;
use ferinth::structures::version_structs::DependencyType;
use ferinth::Ferinth;
use furse::{structures::file_structs::FileRelationType, Furse};
use libium::{
    add,
    config::{self, structs::ModIdentifier},
    upgrade,
};
use octocrab::repos::RepoHandler;
use std::sync::Arc;

pub async fn github(
    repo_handler: RepoHandler<'_>,
    profile: &mut config::structs::Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<()> {
    eprint!("Adding mod... ");
    let repo = libium::add::github(
        &repo_handler,
        profile,
        should_check_game_version,
        should_check_mod_loader,
    )
    .await?;
    upgrade::get_latest_compatible_asset(
        &repo_handler.releases().list().send().await?.items,
        &profile.game_version,
        &profile.mod_loader,
        should_check_game_version,
        should_check_mod_loader,
    )
    .ok_or_else(|| anyhow!("Repository does not release mods compatible with your profile"))?;
    println!("{} ({})", *TICK, repo.name);
    Ok(())
}

pub async fn modrinth(
    modrinth: Arc<Ferinth>,
    project_id: &str,
    profile: &mut config::structs::Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<()> {
    eprint!("Adding mod... ");
    let project = add::modrinth(
        modrinth.clone(),
        project_id,
        profile,
        should_check_game_version,
        should_check_mod_loader,
    )
    .await?;
    let latest_version = upgrade::get_latest_compatible_version(
        &modrinth.list_versions(&project.id).await?,
        &profile.game_version,
        &profile.mod_loader,
        should_check_game_version,
        should_check_mod_loader,
    )
    .ok_or_else(|| anyhow!("Mod is not compatible with your profile"))?
    .1;
    println!("{} ({})", *TICK, project.title);
    for dependency in &latest_version.dependencies {
        let id = if let Some(project_id) = &dependency.project_id {
            project_id.clone()
        } else if let Some(version_id) = &dependency.version_id {
            modrinth.get_version(version_id).await?.project_id
        } else {
            break;
        };
        // If it's required, add it without asking
        if dependency.dependency_type == DependencyType::Required {
            eprint!("Adding required dependency... ");
            match add::modrinth(modrinth.clone(), &id, profile, None, None).await {
                Ok(project) => println!("{} ({})", *TICK, project.title),
                Err(err) => {
                    if matches!(err, add::Error::AlreadyAdded) {
                        println!("{} Already added", *TICK);
                    } else {
                        bail!(err);
                    }
                },
            };
        } else if dependency.dependency_type == DependencyType::Optional {
            let project = modrinth.get_project(&id).await?;
            // If it is not already added:
            if profile.mods.iter().any(|mod_| {
                mod_.name == project.title
                    || ModIdentifier::ModrinthProject(id.clone()) == mod_.identifier
                // And the user wants to add it:
            }) && Confirm::with_theme(&*THEME)
                .with_prompt(format!(
                    "Add optional dependency {} (https://modrinth.com/mod/{})?",
                    project.title, project.slug
                ))
                .interact()?
            {
                eprint!("Adding optional dependency... ");
                let project = add::modrinth(modrinth.clone(), &id, profile, None, None).await?;
                println!("{} ({})", *TICK, project.title);
            }
        }
    }

    Ok(())
}

pub async fn curseforge(
    curseforge: Arc<Furse>,
    project_id: i32,
    profile: &mut config::structs::Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<()> {
    eprint!("Adding mod... ");
    let project = add::curseforge(
        curseforge.clone(),
        project_id,
        profile,
        should_check_game_version,
        should_check_mod_loader,
    )
    .await?;
    let latest_file = upgrade::get_latest_compatible_file(
        curseforge.get_mod_files(project.id).await?,
        &profile.game_version,
        &profile.mod_loader,
        should_check_game_version,
        should_check_mod_loader,
    )
    .ok_or_else(|| anyhow!("Mod is not compatible with your profile"))?
    .0;
    println!("{} ({})", *TICK, project.name);
    for dependency in &latest_file.dependencies {
        let id = dependency.mod_id;
        // If it's required, add it without asking
        if dependency.relation_type == FileRelationType::RequiredDependency {
            eprint!("Adding required dependency... ");
            match add::curseforge(curseforge.clone(), id, profile, None, None).await {
                Ok(project) => println!("{} ({})", *TICK, project.name),
                Err(err) => {
                    if matches!(err, add::Error::AlreadyAdded) {
                        println!("{} Already added", *TICK);
                    } else {
                        bail!(err);
                    }
                },
            };
        } else if dependency.relation_type == FileRelationType::OptionalDependency {
            let project = curseforge.get_mod(id).await?;
            // If it is not already added:
            if !profile.mods.iter().any(|mod_| {
                mod_.name == project.name || ModIdentifier::CurseForgeProject(id) == mod_.identifier
                // And the user wants to add it:
            }) && Confirm::with_theme(&*THEME)
                .with_prompt(format!(
                    "Add optional dependency {} ({})?",
                    project.name, project.links.website_url
                ))
                .interact()?
            {
                eprint!("Adding optional dependency... ");
                let project = add::curseforge(curseforge.clone(), id, profile, None, None).await?;
                println!("{} ({})", *TICK, project.name);
            }
        }
    }

    Ok(())
}
