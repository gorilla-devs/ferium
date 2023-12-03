// CurseForge IDs shouldn't be seperated
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unused_async)]

use anyhow::{bail, Result};
use async_recursion::async_recursion;
use colored::Colorize;
use dialoguer::Confirm;
use ferinth::structures::version::{Dependency, DependencyType};
use ferinth::Ferinth;
use ferinth::structures::project::Project;
use furse::structures::file_structs::FileDependency;
use furse::{structures::file_structs::FileRelationType, Furse};
use itertools::Itertools;
use libium::{
    add,
    config::structs::{Mod, ModIdentifier, ModLoader, Profile},
};
use octocrab::repos::RepoHandler;

use crate::{cli::DependencyLevel, CROSS, THEME, TICK};

struct RecursiveResolver {}
impl RecursiveResolver {
    #[async_recursion]
    async fn modrinth(
        modrinth: &Ferinth,
        profile: &mut Profile,
        should_check_game_version: Option<bool>,
        should_check_mod_loader: Option<bool>,
        project: &Project,
        dep_level: Option<DependencyLevel>,
        dependencies: &Vec<Dependency>,
    ) -> Result<()> {
        for dependency in dependencies {
            let mut id = if let Some(project_id) = &dependency.project_id {
                project_id.clone()
            } else if let Some(version_id) = &dependency.version_id {
                modrinth.get_version(version_id).await?.project_id
            } else {
                break;
            };

            if profile.mod_loader == ModLoader::Quilt {
                if id == "P7dR8mSH" {
                    id = "qvIfYCYJ".into();
                }
                if id == "Ha28R6CL" {
                    id = "lwVhp9o5".into();
                }
            }

            if dependency.dependency_type == DependencyType::Required {
                eprint!(
                    "Adding required dependency {} for mod {}... ",
                    id.to_string().dimmed(),
                    project.title.dimmed()
                );
                let mod_ = modrinth.get_project(&id).await?;
                let latest_file = add::modrinth(modrinth, &mod_, profile, None, None).await;
                match latest_file {
                    Ok(file) => {
                        println!("{} {}", *TICK, mod_.title.bold());
                        profile.mods.push(Mod {
                            name: mod_.title.trim().into(),
                            identifier: ModIdentifier::ModrinthProject(mod_.id),
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
                        if !file.dependencies.is_empty() {
                            Self::modrinth(
                                modrinth,
                                profile,
                                should_check_mod_loader,
                                should_check_game_version,
                                project,
                                dep_level.clone(),
                                &file.dependencies,
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
            } else if dependency.dependency_type == DependencyType::Optional
                && (dep_level == Some(DependencyLevel::All) || dep_level.is_none())
            {
                if dep_level == Some(DependencyLevel::All) {
                    eprint!(
                        "Adding optional dependency {} for mod {}... ",
                        id.to_string().dimmed(),
                        project.title.bold()
                    );
                } else {
                    eprint!(
                        "Checking optional dependency {} for mod {}... ",
                        id.to_string().dimmed(),
                        project.title.bold()
                    );
                }
                let project_local = modrinth.get_project(&id).await?;
                let project_local_clone = project_local.clone();
                match add::modrinth(modrinth, &project_local, profile, None, None).await {
                    Ok(file) => {
                        if dep_level.is_none() {
                            println!("{}", *TICK);
                        }
                        // If it's optional, confirm with the user if they want to add it
                        if dep_level == Some(DependencyLevel::All)
                            || Confirm::with_theme(&*THEME)
                                .with_prompt(format!(
                                    "Add optional dependency {} for mod {} ({})?",
                                    project_local.title.bold(),
                                    project.title.bold(),
                                    format!("https://modrinth.com/mod/{}", project_local.slug)
                                        .blue()
                                        .underline()
                                ))
                                .interact()?
                        {
                            profile.mods.push(Mod {
                                name: project_local.title.trim().into(),
                                identifier: ModIdentifier::ModrinthProject(project_local.id),
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
                            if !file.dependencies.is_empty() {
                                Self::modrinth(
                                    modrinth,
                                    profile,
                                    should_check_mod_loader,
                                    should_check_game_version,
                                    &project_local_clone,
                                    dep_level.clone(),
                                    &file.dependencies,
                                )
                                .await?;
                            }
                            if dep_level == Some(DependencyLevel::All) {
                                println!("{} {}", *TICK, project_local.title.bold());
                            }
                        }
                    }
                    Err(err) => {
                        if matches!(err, add::Error::AlreadyAdded) {
                            println!("{} Already added", *TICK);
                        } else {
                            println!("{}", format!("{} {}", CROSS, err).yellow());
                        }
                    }
                };
            }
        }

        Ok(())
    }

    #[async_recursion]
    async fn curseforge(
        curseforge: &Furse,
        profile: &mut Profile,
        should_check_mod_loader: Option<bool>,
        should_check_game_version: Option<bool>,
        original_mod: &furse::structures::mod_structs::Mod,
        dep_level: Option<DependencyLevel>,
        dependencies: Vec<FileDependency>,
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
                    "Adding required dependency {} for mod {}... ",
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
                        if !file.dependencies.is_empty() {
                            Self::curseforge(
                                curseforge,
                                profile,
                                should_check_mod_loader,
                                should_check_game_version,
                                original_mod,
                                dep_level.clone(),
                                file.dependencies,
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
                    eprint!(
                        "Adding optional dependency {} for mod {}... ",
                        id.to_string().dimmed(),
                        original_mod.name.bold()
                    );
                } else {
                    eprint!(
                        "Checking optional dependency {} for mod {}... ",
                        id.to_string().dimmed(),
                        original_mod.name.bold()
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
                                    "Add optional dependency {} for mod {} ({})?",
                                    project.name.bold(),
                                    original_mod.name.bold(),
                                    project.links.website_url.to_string().blue().underline()
                                ))
                                .interact()?
                        {
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
                            if !file.dependencies.is_empty() {
                                Self::curseforge(
                                    curseforge,
                                    profile,
                                    should_check_mod_loader,
                                    should_check_game_version,
                                    original_mod,
                                    dep_level.clone(),
                                    file.dependencies,
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
                            println!("{}", format!("{} {}", CROSS, err).yellow());
                        }
                    }
                };
            }
        }

        Ok(())
    }
}

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
    dependencies: Option<DependencyLevel>,
) -> Result<()> {
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
        identifier: ModIdentifier::ModrinthProject(project.clone().id),
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
    if !latest_version.dependencies.is_empty() {
        RecursiveResolver::modrinth(
            modrinth,
            profile,
            should_check_game_version,
            should_check_mod_loader,
            &project,
            dependencies,
            &latest_version.dependencies,
        )
        .await?;
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
        RecursiveResolver::curseforge(
            curseforge,
            profile,
            should_check_mod_loader,
            should_check_game_version,
            &project,
            dependencies,
            latest_file.dependencies,
        )
        .await?;
    }

    Ok(())
}
