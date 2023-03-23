use crate::{cli::DependencyLevel, CROSS, THEME, TICK};
use anyhow::{bail, Result};
use colored::Colorize;
use dialoguer::Confirm;
use ferinth::structures::version::DependencyType;
use ferinth::Ferinth;
use furse::{structures::file_structs::FileRelationType, Furse};
use itertools::Itertools;
use libium::{
    add,
    config::structs::{Mod, ModIdentifier, ModLoader, Profile},
};
use octocrab::repos::RepoHandler;

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

            // Replace Fabric API with Quilted Fabric API on Quilt
            if profile.mod_loader == ModLoader::Quilt && id == "P7dR8mSH" {
                id = "qvIfYCYJ".into();
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
                            println!("{}", format!("{CROSS} {err}").yellow());
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
        for dependency in &latest_file.dependencies {
            let id = dependency.mod_id;
            if dependency.relation_type == FileRelationType::RequiredDependency {
                eprint!("Adding required dependency {}... ", id.to_string().dimmed());
                let project = curseforge.get_mod(id).await?;
                match add::curseforge(curseforge, &project, profile, None, None).await {
                    Ok(_) => {
                        println!("{} {}", *TICK, project.name.bold());
                        // If it's required, add it without asking
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
                    }
                    Err(err) => {
                        if matches!(err, add::Error::AlreadyAdded) {
                            println!("{} Already added", *TICK);
                        } else {
                            bail!(err);
                        }
                    }
                };
            } else if dependency.relation_type == FileRelationType::OptionalDependency
                && (dependencies == Some(DependencyLevel::All) || dependencies.is_none())
            {
                if dependencies == Some(DependencyLevel::All) {
                    eprint!("Adding optional dependency {}... ", id.to_string().dimmed());
                } else {
                    eprint!(
                        "Checking optional dependency {}... ",
                        id.to_string().dimmed()
                    );
                }
                let project = curseforge.get_mod(id).await?;
                match add::curseforge(curseforge, &project, profile, None, None).await {
                    Ok(_) => {
                        if dependencies.is_none() {
                            println!("{}", *TICK);
                        }
                        // If it's optional, confirm with the user if they want to add it
                        if dependencies == Some(DependencyLevel::All)
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
                                println!("{} {}", *TICK, project.name.bold());
                            }
                        }
                    }
                    Err(err) => {
                        if matches!(err, add::Error::AlreadyAdded) {
                            println!("{} Already added", *TICK);
                        } else {
                            println!("{}", format!("{CROSS} {err}").yellow());
                        }
                    }
                };
            }
        }
    }

    Ok(())
}
