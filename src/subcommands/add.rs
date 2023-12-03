// CurseForge IDs shouldn't be seperated
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unused_async)]

use anyhow::{bail, Result};
use async_recursion::async_recursion;
use colored::Colorize;
use dialoguer::Confirm;
use ferinth::structures::version::DependencyType;
use ferinth::Ferinth;
use furse::structures::file_structs::FileDependency;
use furse::{structures::file_structs::FileRelationType, Furse};
use itertools::Itertools;
use libium::{
    add,
    config::structs::{Mod, ModIdentifier, ModLoader, Profile},
};
use octocrab::repos::RepoHandler;

use crate::{cli::DependencyLevel, CROSSSIGN, THEME, TICK};

#[allow(clippy::expect_used)]
pub async fn github(
    repo_handler: RepoHandler<'_>,
    profile: &mut Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<()> {
    eprint!("Adding mod... ");
    let (repo, _) = add::github(
        &repo_handler,
        profile,
        should_check_game_version,
        should_check_mod_loader,
    )
    .await?;
    println!("{} {}", *TICK, repo.name.bold());
    profile.mods.push(Mod {
        name: repo.name.trim().into(),
        identifier: ModIdentifier::GitHubRepository((
            repo.owner.expect("Could not get repository owner").login,
            repo.name,
        )),
        check_game_version: if should_check_game_version == Some(true) {
            None
        } else {
            should_check_game_version
        },
        check_mod_loader: if should_check_mod_loader == Some(true) {
            None
        } else {
            should_check_mod_loader
        },
    });
    Ok(())
}

pub async fn modrinth(
    modrinth: &Ferinth,
    project_id: &str,
    profile: &mut Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<()> {
    let dependencies = None;
    eprint!("Adding mod... ");
    let project = modrinth.get_project(project_id).await?;
    let latest_version = add::modrinth(
        modrinth,
        &project,
        profile,
        should_check_game_version,
        should_check_mod_loader,
    )
    .await?;
    println!("{} {}", *TICK, project.title.bold());
    profile.mods.push(Mod {
        name: project.title.trim().into(),
        identifier: ModIdentifier::ModrinthProject(project.id),
        check_game_version: if should_check_game_version == Some(true) {
            None
        } else {
            should_check_game_version
        },
        check_mod_loader: if should_check_mod_loader == Some(true) {
            None
        } else {
            should_check_mod_loader
        },
    });
    if dependencies != Some(DependencyLevel::None) {
        for dependency in &latest_version.dependencies {
            let mut id = if let Some(project_id) = &dependency.project_id {
                project_id.clone()
            } else if let Some(version_id) = &dependency.version_id {
                modrinth.get_version(version_id).await?.project_id
            } else {
                break;
            };

            if profile.mod_loader == ModLoader::Quilt {
                // Fabric API
                if id == "P7dR8mSH" {
                    // Quilted Fabric API
                    id = "qvIfYCYJ".into();
                }
                // Fabric Language Kotlin
                if id == "Ha28R6CL" {
                    // Quilt Kotlin Libraries
                    id = "lwVhp9o5".into();
                }
            }

            if dependency.dependency_type == DependencyType::Required {
                eprint!("Adding required dependency {}... ", id.dimmed());
                let project = modrinth.get_project(&id).await?;
                match add::modrinth(modrinth, &project, profile, None, None).await {
                    Ok(_) => {
                        println!("{} {}", *TICK, project.title.bold());
                        // If it's required, add it without asking
                        profile.mods.push(Mod {
                            name: project.title.trim().into(),
                            identifier: ModIdentifier::ModrinthProject(project.id),
                            check_game_version: if should_check_game_version == Some(true) {
                                None
                            } else {
                                should_check_game_version
                            },
                            check_mod_loader: if should_check_mod_loader == Some(true) {
                                None
                            } else {
                                should_check_mod_loader
                            },
                        });
                    }
                    Err(err) => {
                        if matches!(err, add::Error::AlreadyAdded) {
                            println!("{} Already added", *TICK);
                        } else {
                            bail!(err);
                        }
                    }
                };
            } else if dependency.dependency_type == DependencyType::Optional
                && (dependencies == Some(DependencyLevel::All) || dependencies.is_none())
            {
                if dependencies == Some(DependencyLevel::All) {
                    eprint!("Adding optional dependency {}... ", id.dimmed());
                } else {
                    eprint!("Checking optional dependency {}... ", id.dimmed());
                }
                let project = modrinth.get_project(&id).await?;
                match add::modrinth(modrinth, &project, profile, None, None).await {
                    Ok(_) => {
                        if dependencies.is_none() {
                            println!("{}", *TICK);
                        }
                        // If it's optional, confirm with the user if they want to add it
                        if dependencies == Some(DependencyLevel::All)
                            || Confirm::with_theme(&*THEME)
                                .with_prompt(format!(
                                    "Add optional dependency {} ({})?",
                                    project.title.bold(),
                                    format!("https://modrinth.com/mod/{}", project.slug)
                                        .blue()
                                        .underline()
                                ))
                                .interact()?
                        {
                            profile.mods.push(Mod {
                                name: project.title.trim().into(),
                                identifier: ModIdentifier::ModrinthProject(project.id),
                                check_game_version: if should_check_game_version == Some(true) {
                                    None
                                } else {
                                    should_check_game_version
                                },
                                check_mod_loader: if should_check_mod_loader == Some(true) {
                                    None
                                } else {
                                    should_check_mod_loader
                                },
                            });
                            if dependencies == Some(DependencyLevel::All) {
                                println!("{} {}", *TICK, project.title.bold());
                            }
                        }
                    }
                    Err(err) => {
                        if matches!(err, add::Error::AlreadyAdded) {
                            println!("{} Already added", *TICK);
                        } else {
                            println!("{}", format!("{CROSSSIGN} {err}").yellow());
                        }
                    }
                };
            }
        }
    }

    if !project.donation_urls.is_empty() {
        println!(
            "Consider supporting the mod creator on {}",
            project
                .donation_urls
                .iter()
                .map(|this| format!(
                    "{} ({})",
                    this.platform.bold(),
                    this.url.to_string().blue().underline()
                ))
                .format(" or ")
        );
    }

    Ok(())
}

#[async_recursion]
async fn resolve_deps(
    curseforge: &Furse,
    dependencies: Vec<FileDependency>,
    profile: &mut Profile,
    original_mod: &furse::structures::mod_structs::Mod,
    dep_level: Option<DependencyLevel>,
) -> Result<()> {
    for dependency in &dependencies {
        let mut id = dependency.mod_id;
        if profile.mod_loader == ModLoader::Quilt {
            if id == 306612 {
                id = 634179;
            }
            if id == 308769 {
                id = 720410;
            }
        }
        if dependency.relation_type == FileRelationType::RequiredDependency {
            eprint!(
                "Adding required dependency {} of mod {}... ",
                id.to_string().dimmed(),
                original_mod.name
            );
            let mod_ = curseforge.get_mod(id).await?;
            let latest_file = add::curseforge(curseforge, &mod_, profile, None, None).await;
            match latest_file {
                Ok(file) => {
                    println!("{} {}", *TICK, mod_.name.bold());
                    profile.mods.push(Mod {
                        name: mod_.name.trim().into(),
                        identifier: ModIdentifier::CurseForgeProject(mod_.id),
                        check_game_version: None,
                        check_mod_loader: None,
                    });
                    if !file.dependencies.is_empty() {
                        resolve_deps(
                            curseforge,
                            file.dependencies,
                            profile,
                            &mod_,
                            dep_level.clone(),
                        )
                        .await?;
                    }
                }
                Err(err) => {
                    if matches!(err, add::Error::AlreadyAdded) {
                        println!("{} Already added", *TICK);
                    } else {
                        bail!(err);
                    }
                }
            }
        } else if dependency.relation_type == FileRelationType::OptionalDependency
            && (dep_level == Some(DependencyLevel::All) || dep_level.is_none())
        {
            if dep_level == Some(DependencyLevel::All) {
                eprint!("Adding optional dependency {}... ", id.to_string().dimmed());
            } else {
                eprint!(
                    "Checking optional dependency {}... ",
                    id.to_string().dimmed()
                );
            }
            let project = curseforge.get_mod(id).await?;
            match add::curseforge(curseforge, &project, profile, None, None).await {
                Ok(file) => {
                    if dep_level.is_none() {
                        println!("{}", *TICK);
                    }
                    // If it's optional, confirm with the user if they want to add it
                    if dep_level == Some(DependencyLevel::All)
                        || Confirm::with_theme(&*THEME)
                            .with_prompt(format!(
                                "Add optional dependency {} ({})?",
                                project.name.bold(),
                                project.links.website_url.to_string().blue().underline()
                            ))
                            .interact()?
                    {
                        profile.mods.push(Mod {
                            name: project.name.trim().into(),
                            identifier: ModIdentifier::CurseForgeProject(project.id),
                            check_game_version: None,
                            check_mod_loader: None,
                        });
                        if !file.dependencies.is_empty() {
                            resolve_deps(
                                curseforge,
                                file.dependencies,
                                profile,
                                &project,
                                dep_level.clone(),
                            )
                            .await?;
                        }
                        if dep_level == Some(DependencyLevel::All) {
                            println!("{} {}", *TICK, project.name.bold());
                        }
                    }
                }
                Err(err) => {
                    if matches!(err, add::Error::AlreadyAdded) {
                        println!("{} Already added", *TICK);
                    } else {
                        println!("{}", format!("{CROSSSIGN} {err}").yellow());
                    }
                }
            };
        }
    }

    Ok(())
}
pub async fn curseforge(
    curseforge: &Furse,
    project_id: i32,
    profile: &mut Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
    dependencies: Option<DependencyLevel>,
) -> Result<()> {
    eprint!("Adding mod... ");
    let project = curseforge.get_mod(project_id).await?;
    let latest_file = add::curseforge(
        curseforge,
        &project,
        profile,
        should_check_game_version,
        should_check_mod_loader,
    )
    .await?;

    println!("{} {}", *TICK, project.name.bold());
    profile.mods.push(Mod {
        name: project.name.trim().into(),
        identifier: ModIdentifier::CurseForgeProject(project.id),
        check_game_version: if should_check_game_version == Some(true) {
            None
        } else {
            should_check_game_version
        },
        check_mod_loader: if should_check_mod_loader == Some(true) {
            None
        } else {
            should_check_mod_loader
        },
    });

    if dependencies != Some(DependencyLevel::None) {
        resolve_deps(
            curseforge,
            latest_file.dependencies,
            profile,
            &project,
            dependencies,
        )
        .await?;
    }

    Ok(())
}
