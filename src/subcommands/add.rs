use crate::{CROSS, THEME, TICK};
use anyhow::{bail, Result};
use colored::Colorize;
use dialoguer::Confirm;
use ferinth::{structures::version_structs::DependencyType, Ferinth};
use furse::{structures::file_structs::FileRelationType, Furse};
use itertools::Itertools;
use libium::{
    add,
    config::structs::{Mod, ModIdentifier, Profile},
};
use octocrab::repos::RepoHandler;
use std::sync::Arc;

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
        name:               repo.name.trim().into(),
        identifier:         ModIdentifier::GitHubRepository((
            repo.owner.expect("Could not get repository owner").login,
            repo.name,
        )),
        check_game_version: if should_check_game_version == Some(true) {
            None
        } else {
            should_check_game_version
        },
        check_mod_loader:   if should_check_mod_loader == Some(true) {
            None
        } else {
            should_check_mod_loader
        },
    });
    Ok(())
}

pub async fn modrinth(
    modrinth: &Arc<Ferinth>,
    project_id: &str,
    profile: &mut Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
    add_dependencies: bool,
) -> Result<()> {
    eprint!("Adding mod... ");
    let (project, latest_version) = add::modrinth(
        modrinth.clone(),
        project_id,
        profile,
        should_check_game_version,
        should_check_mod_loader,
    )
    .await?;
    println!("{} {}", *TICK, project.title.bold());
    profile.mods.push(Mod {
        name:               project.title.trim().into(),
        identifier:         ModIdentifier::ModrinthProject(project.id),
        check_game_version: if should_check_game_version == Some(true) {
            None
        } else {
            should_check_game_version
        },
        check_mod_loader:   if should_check_mod_loader == Some(true) {
            None
        } else {
            should_check_mod_loader
        },
    });
    if add_dependencies {
        for dependency in &latest_version.dependencies {
            let id = if let Some(project_id) = &dependency.project_id {
                project_id.clone()
            } else if let Some(version_id) = &dependency.version_id {
                modrinth.get_version(version_id).await?.project_id
            } else {
                break;
            };
            if dependency.dependency_type == DependencyType::Required {
                eprint!("Adding required dependency... ");
                match add::modrinth(modrinth.clone(), &id, profile, None, None).await {
                    Ok((project, _)) => {
                        println!("{} {}", *TICK, project.title.bold());
                        // If it's required, add it without asking
                        profile.mods.push(Mod {
                            name:               project.title.trim().into(),
                            identifier:         ModIdentifier::ModrinthProject(project.id),
                            check_game_version: if should_check_game_version == Some(true) {
                                None
                            } else {
                                should_check_game_version
                            },
                            check_mod_loader:   if should_check_mod_loader == Some(true) {
                                None
                            } else {
                                should_check_mod_loader
                            },
                        });
                    },
                    Err(err) => {
                        if matches!(err, add::Error::AlreadyAdded) {
                            println!("{} Already added", *TICK);
                        } else {
                            bail!(err);
                        }
                    },
                };
            } else if dependency.dependency_type == DependencyType::Optional {
                eprint!("Checking optional dependency... ");
                match add::modrinth(modrinth.clone(), &id, profile, None, None).await {
                    Ok((project, _)) => {
                        println!("{}", *TICK);
                        // If it's optional, confirm with the user if they want to add it
                        if Confirm::with_theme(&*THEME)
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
                                name:               project.title.trim().into(),
                                identifier:         ModIdentifier::ModrinthProject(project.id),
                                check_game_version: if should_check_game_version == Some(true) {
                                    None
                                } else {
                                    should_check_game_version
                                },
                                check_mod_loader:   if should_check_mod_loader == Some(true) {
                                    None
                                } else {
                                    should_check_mod_loader
                                },
                            });
                        }
                    },
                    Err(err) => {
                        if matches!(err, add::Error::AlreadyAdded) {
                            println!("{} Already added", *TICK);
                        } else {
                            println!("{}", format!("{} {}", CROSS, err).yellow());
                        }
                    },
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

pub async fn curseforge(
    curseforge: &Arc<Furse>,
    project_id: i32,
    profile: &mut Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
    add_dependencies: bool,
) -> Result<()> {
    eprint!("Adding mod... ");
    let (project, latest_file) = add::curseforge(
        curseforge.clone(),
        project_id,
        profile,
        should_check_game_version,
        should_check_mod_loader,
    )
    .await?;
    println!("{} {}", *TICK, project.name.bold());
    profile.mods.push(Mod {
        name:               project.name.trim().into(),
        identifier:         ModIdentifier::CurseForgeProject(project.id),
        check_game_version: if should_check_game_version == Some(true) {
            None
        } else {
            should_check_game_version
        },
        check_mod_loader:   if should_check_mod_loader == Some(true) {
            None
        } else {
            should_check_mod_loader
        },
    });
    if add_dependencies {
        for dependency in &latest_file.dependencies {
            let id = dependency.mod_id;
            if dependency.relation_type == FileRelationType::RequiredDependency {
                eprint!("Adding required dependency... ");
                match add::curseforge(curseforge.clone(), id, profile, None, None).await {
                    Ok((project, _)) => {
                        println!("{} {}", *TICK, project.name.bold());
                        // If it's required, add it without asking
                        profile.mods.push(Mod {
                            name:               project.name.trim().into(),
                            identifier:         ModIdentifier::CurseForgeProject(project.id),
                            check_game_version: if should_check_game_version == Some(true) {
                                None
                            } else {
                                should_check_game_version
                            },
                            check_mod_loader:   if should_check_mod_loader == Some(true) {
                                None
                            } else {
                                should_check_mod_loader
                            },
                        });
                    },
                    Err(err) => {
                        if matches!(err, add::Error::AlreadyAdded) {
                            println!("{} Already added", *TICK);
                        } else {
                            bail!(err);
                        }
                    },
                };
            } else if dependency.relation_type == FileRelationType::OptionalDependency {
                eprint!("Checking optional dependency... ");
                match add::curseforge(curseforge.clone(), id, profile, None, None).await {
                    Ok((project, _)) => {
                        println!("{}", *TICK);
                        // If it's optional, confirm with the user if they want to add it
                        if Confirm::with_theme(&*THEME)
                            .with_prompt(format!(
                                "Add optional dependency {} ({})?",
                                project.name.bold(),
                                project.links.website_url.to_string().blue().underline()
                            ))
                            .interact()?
                        {
                            profile.mods.push(Mod {
                                name:               project.name.trim().into(),
                                identifier:         ModIdentifier::CurseForgeProject(project.id),
                                check_game_version: if should_check_game_version == Some(true) {
                                    None
                                } else {
                                    should_check_game_version
                                },
                                check_mod_loader:   if should_check_mod_loader == Some(true) {
                                    None
                                } else {
                                    should_check_mod_loader
                                },
                            });
                        }
                    },
                    Err(err) => {
                        if matches!(err, add::Error::AlreadyAdded) {
                            println!("{} Already added", *TICK);
                        } else {
                            println!("{}", format!("{} {}", CROSS, err).yellow());
                        }
                    },
                };
            }
        }
    }

    Ok(())
}
