use crate::{THEME, TICK};
use anyhow::Result;
use dialoguer::Confirm;
use ferinth::structures::version_structs::DependencyType;
use ferinth::Ferinth;
use furse::{structures::file_structs::FileRelationType, Furse};
use libium::{add, config};

pub async fn modrinth(
    modrinth: &Ferinth,
    profile: &mut config::structs::Profile,
    project_id: &str,
) -> Result<()> {
    eprint!("Adding mod... ");
    let project = add::modrinth(modrinth, project_id, profile).await?;
    // Add dependencies
    let latest_version =
        libium::upgrade::modrinth(modrinth, profile, &project.id, Some(true), Some(true))
            .await?
            .0;
    println!("{} ({})", *TICK, project.title);
    for dependency in latest_version.dependencies {
        let id = if let Some(project_id) = dependency.project_id {
            project_id
        } else if let Some(version_id) = dependency.version_id {
            modrinth.get_version(&version_id).await?.project_id
        } else {
            break;
        };
        // Check if the dependency has already been added
        if !profile.mods.iter().any(|mod_| {
            config::structs::ModIdentifier::ModrinthProject(id.clone()) == mod_.identifier
        }) {
            // If it's required, add it without asking
            if dependency.dependency_type == DependencyType::Required {
                eprint!("Adding required dependency... ");
                let project = add::modrinth(modrinth, &id, profile).await?;
                println!("{} ({})", *TICK, project.title);
            } else if dependency.dependency_type == DependencyType::Optional {
                let project = modrinth.get_project(&id).await?;
                let should_add = Confirm::with_theme(&*THEME)
                    .with_prompt(format!(
                        "Add optional dependency {} (https://modrinth.com/mod/{})?",
                        project.title, project.slug
                    ))
                    .interact()?;
                if should_add {
                    eprint!("Adding optional dependency... ");
                    let project = add::modrinth(modrinth, &id, profile).await?;
                    println!("{} ({})", *TICK, project.title);
                }
            }
        }
    }

    Ok(())
}

pub async fn curseforge(
    curseforge: &Furse,
    profile: &mut config::structs::Profile,
    project_id: i32,
) -> Result<()> {
    eprint!("Adding mod... ");
    let project = add::curseforge(curseforge, project_id, profile).await?;
    // Add dependencies
    let latest_version =
        libium::upgrade::curseforge(curseforge, profile, project.id, Some(true), Some(true))
            .await?
            .0;
    println!("{} ({})", *TICK, project.name);
    for dependency in latest_version.dependencies {
        let id = dependency.mod_id;
        // Check if the dependency has already been added
        if !profile
            .mods
            .iter()
            .any(|mod_| config::structs::ModIdentifier::CurseForgeProject(id) == mod_.identifier)
        {
            // If it's required, add it without asking
            if dependency.relation_type == FileRelationType::RequiredDependency {
                eprint!("Adding required dependency... ");
                let project = add::curseforge(curseforge, id, profile).await?;
                println!("{} ({})", *TICK, project.name);
            } else if dependency.relation_type == FileRelationType::OptionalDependency {
                let project = curseforge.get_mod(id).await?;
                let should_add = Confirm::with_theme(&*THEME)
                    .with_prompt(format!(
                        "Add optional dependency {} ({})?",
                        project.name, project.links.website_url
                    ))
                    .interact()?;
                if should_add {
                    eprint!("Adding optional dependency... ");
                    let project = add::curseforge(curseforge, id, profile).await?;
                    println!("{} ({})", *TICK, project.name);
                }
            }
        }
    }

    Ok(())
}
