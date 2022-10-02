mod cli;
mod download;
mod subcommands;

use anyhow::{anyhow, bail, Result};
use clap::{CommandFactory, Parser};
use cli::{Ferium, ModpackSubCommands, ProfileSubCommands, SubCommands};
use colored::{ColoredString, Colorize};
use dialoguer::theme::ColorfulTheme;
use ferinth::Ferinth;
use furse::Furse;
use indicatif::ProgressStyle;
use lazy_static::lazy_static;
use libium::config::{
    self,
    structs::{Config, ModIdentifier, Modpack, Profile},
};
use octocrab::OctocrabBuilder;
use online::tokio::check;
use std::{
    env::{var, var_os},
    process::ExitCode,
    sync::Arc,
};
use tokio::{runtime, spawn};

const CROSS: &str = "×";
lazy_static! {
    pub static ref TICK: ColoredString = "✓".green();
    pub static ref YELLOW_TICK: ColoredString = "✓".yellow();
    pub static ref THEME: ColorfulTheme = ColorfulTheme::default();
}

#[allow(clippy::expect_used)]
pub fn style_no() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("{spinner} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:.cyan}/{len:.blue}")
        .expect("Progess bar template parse failure")
        .progress_chars("#>-")
}

#[allow(clippy::expect_used)]
pub fn style_byte() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template(
            "{spinner} [{bytes_per_sec}] [{wide_bar:.cyan/blue}] {bytes:.cyan}/{total_bytes:.blue}",
        )
        .expect("Progess bar template parse failure")
        .progress_chars("#>-")
}

fn main() -> ExitCode {
    let cli = Ferium::parse();
    let mut builder = runtime::Builder::new_multi_thread();
    builder.enable_all();
    builder.thread_name("ferium-worker");
    if let Some(threads) = cli.threads {
        builder.max_blocking_threads(threads);
    }
    #[allow(clippy::expect_used)] // No error handling yet
    let runtime = builder.build().expect("Could not initialise Tokio runtime");
    if let Err(err) = runtime.block_on(actual_main(cli)) {
        eprintln!("{}", err.to_string().red().bold());
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

#[allow(clippy::future_not_send)] // 3rd party library doesn't implement `Send`
async fn actual_main(cli_app: Ferium) -> Result<()> {
    // The complete command should not require a config.
    // See [#139](https://github.com/gorilla-devs/ferium/issues/139) for why this might be a problem.
    if let SubCommands::Complete { shell } = cli_app.subcommand {
        clap_complete::generate(
            shell,
            &mut Ferium::command(),
            "ferium",
            &mut std::io::stdout(),
        );
        return Ok(());
    }

    let github = Arc::new(
        cli_app
            .github_token
            .map_or_else(OctocrabBuilder::new, |token| {
                OctocrabBuilder::new().personal_token(token)
            })
            .build()?,
    );
    let modrinth = Arc::new(Ferinth::new(
        "ferium",
        option_env!("CARGO_PKG_VERSION"),
        Some("theRookieCoder#1287"),
        None,
    )?);
    // Yes this is a personal API key, but I am allowed to write it in source.
    // The reason is the API key is used for tracking usage, it's not for authentication.
    // So please don't use this outside of Ferium, although telling you not to is all I can do...
    let curseforge = Arc::new(Furse::new(&var("CURSEFORGE_API_KEY").unwrap_or_else(
        |_| "$2a$10$QbCxI6f4KxEs50QKwE2piu1t6oOA8ayOw27H9N/eaH3Sdp5NTWwvO".into(),
    )));

    let mut config_file = config::get_file(
        cli_app
            .config_file
            .or_else(|| var_os("FERIUM_CONFIG_FILE").map(Into::into))
            .unwrap_or_else(config::file_path),
    )
    .await?;
    let mut config = config::deserialise(&config::read_file(&mut config_file).await?)?;

    // Run function(s) based on the sub(sub)command to be executed
    match cli_app.subcommand {
        SubCommands::Complete { .. } => unreachable!(),
        SubCommands::Add {
            identifier,
            dont_check_game_version,
            dont_check_mod_loader,
            dependencies,
        } => {
            let profile = get_active_profile(&mut config)?;
            check_internet().await?;
            if let Ok(project_id) = identifier.parse::<i32>() {
                subcommands::add::curseforge(
                    curseforge,
                    project_id,
                    profile,
                    Some(!dont_check_game_version),
                    Some(!dont_check_mod_loader),
                    dependencies,
                )
                .await?;
            } else if identifier.split('/').count() == 2 {
                let split = identifier.split('/').collect::<Vec<_>>();
                subcommands::add::github(
                    github.repos(split[0], split[1]),
                    profile,
                    Some(!dont_check_game_version),
                    Some(!dont_check_mod_loader),
                )
                .await?;
            } else if let Err(err) = subcommands::add::modrinth(
                modrinth,
                &identifier,
                profile,
                Some(!dont_check_game_version),
                Some(!dont_check_mod_loader),
                dependencies,
            )
            .await
            {
                return Err(
                    if err.to_string() == ferinth::Error::NotBase62.to_string() {
                        anyhow!("Invalid indentifier")
                    } else {
                        err
                    },
                );
            }
        },
        SubCommands::List { verbose, markdown } => {
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile)?;
            if verbose {
                check_internet().await?;
                let mut tasks = Vec::new();
                for mod_ in &profile.mods {
                    if markdown {
                        match &mod_.identifier {
                            ModIdentifier::CurseForgeProject(project_id) => {
                                subcommands::list::curseforge_md(curseforge.clone(), *project_id)
                                    .await?;
                            },
                            ModIdentifier::ModrinthProject(project_id) => {
                                subcommands::list::modrinth_md(
                                    modrinth.clone(),
                                    project_id.clone(),
                                )
                                .await?;
                            },
                            ModIdentifier::GitHubRepository(full_name) => {
                                subcommands::list::github_md(github.clone(), full_name.clone())
                                    .await?;
                            },
                        };
                    } else {
                        let mut mr_ids = Vec::<&str>::new();
                        match &mod_.identifier {
                            ModIdentifier::CurseForgeProject(project_id) => tasks.push(spawn(
                                subcommands::list::curseforge(curseforge.clone(), *project_id),
                            )),
                            ModIdentifier::ModrinthProject(project_id) => mr_ids.push(project_id),
                            ModIdentifier::GitHubRepository(full_name) => tasks.push(spawn(
                                subcommands::list::github(github.clone(), full_name.clone()),
                            )),
                        };
                        let mr_projects = modrinth.get_multiple_projects(&mr_ids).await?;
                        let mr_teams_members = modrinth
                            .list_multiple_teams_members(
                                &mr_projects
                                    .iter()
                                    .map(|p| &p.team as &str)
                                    .collect::<Vec<_>>(),
                            )
                            .await?;
                        for (project, team_members) in mr_projects.into_iter().zip(mr_teams_members.into_iter()) {
                            tasks.push(spawn(subcommands::list::modrinth(project, team_members)));
                        }
                    }
                }
                for handle in tasks {
                    handle.await??;
                }
            } else {
                for mod_ in &profile.mods {
                    println!(
                        "{:45} {}",
                        mod_.name.bold(),
                        match &mod_.identifier {
                            ModIdentifier::CurseForgeProject(id) =>
                                format!("{:10} {}", "CurseForge".red(), id.to_string().dimmed()),
                            ModIdentifier::ModrinthProject(id) =>
                                format!("{:10} {}", "Modrinth".green(), id.dimmed()),
                            ModIdentifier::GitHubRepository(name) => format!(
                                "{:10} {}",
                                "GitHub".purple(),
                                format!("{}/{}", name.0, name.1).dimmed()
                            ),
                        },
                    );
                }
            }
        },
        SubCommands::Modpack { subcommand } => match subcommand {
            ModpackSubCommands::Add {
                identifier,
                output_dir,
                install_overrides,
            } => {
                check_internet().await?;
                if let Ok(project_id) = identifier.parse::<i32>() {
                    subcommands::modpack::add::curseforge(
                        curseforge.clone(),
                        &mut config,
                        project_id,
                        output_dir,
                        install_overrides,
                    )
                    .await?;
                } else if let Err(err) = subcommands::modpack::add::modrinth(
                    modrinth.clone(),
                    &mut config,
                    &identifier,
                    output_dir,
                    install_overrides,
                )
                .await
                {
                    return Err(
                        if err.to_string() == ferinth::Error::NotBase62.to_string() {
                            anyhow!("Invalid indentifier")
                        } else {
                            err
                        },
                    );
                }
            },
            ModpackSubCommands::Configure {
                output_dir,
                install_overrides,
            } => {
                subcommands::modpack::configure(
                    get_active_modpack(&mut config)?,
                    output_dir,
                    install_overrides,
                )
                .await?;
            },
            ModpackSubCommands::Delete { modpack_name } => {
                subcommands::modpack::delete(&mut config, modpack_name)?;
            },
            ModpackSubCommands::List => {
                if config.modpacks.is_empty() {
                    bail!("There are no modpacks configured, add a modpack using `ferium modpack add`")
                }
                subcommands::modpack::list(&config);
            },
            ModpackSubCommands::Switch { modpack_name } => {
                subcommands::modpack::switch(&mut config, modpack_name)?;
            },
            ModpackSubCommands::Upgrade => {
                check_internet().await?;
                subcommands::modpack::upgrade(
                    modrinth.clone(),
                    curseforge.clone(),
                    get_active_modpack(&mut config)?,
                )
                .await?;
            },
        },
        SubCommands::Profile { subcommand } => match subcommand {
            ProfileSubCommands::Configure {
                game_version,
                mod_loader,
                name,
                output_dir,
            } => {
                check_internet().await?;
                subcommands::profile::configure(
                    get_active_profile(&mut config)?,
                    game_version,
                    mod_loader,
                    name,
                    output_dir,
                )
                .await?;
            },
            ProfileSubCommands::Create {
                import,
                game_version,
                mod_loader,
                name,
                output_dir,
            } => {
                if game_version.is_none() {
                    check_internet().await?;
                }
                subcommands::profile::create(
                    &mut config,
                    import,
                    game_version,
                    mod_loader,
                    name,
                    output_dir,
                )
                .await?;
            },
            ProfileSubCommands::Delete { profile_name } => {
                subcommands::profile::delete(&mut config, profile_name)?;
            },
            ProfileSubCommands::List => {
                if config.profiles.is_empty() {
                    bail!("There are no profiles configured, create a profile using `ferium profile create`")
                }
                subcommands::profile::list(&config);
            },
            ProfileSubCommands::Switch { profile_name } => {
                subcommands::profile::switch(&mut config, profile_name)?;
            },
        },
        SubCommands::Remove { mod_names } => {
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile)?;
            subcommands::remove(profile, mod_names)?;
        },
        SubCommands::Upgrade => {
            check_internet().await?;
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile)?;
            subcommands::upgrade(modrinth, curseforge, github, profile).await?;
        },
    };

    config.profiles.iter_mut().for_each(|profile| {
        profile
            .mods
            .sort_by_cached_key(|mod_| mod_.name.to_lowercase());
    });
    // Update config file with possibly edited config
    config::write_file(&mut config_file, &config).await?;

    Ok(())
}

/// Get the active profile with error handling
fn get_active_profile(config: &mut Config) -> Result<&mut Profile> {
    if config.profiles.is_empty() {
        bail!("There are no profiles configured, create a profile using `ferium profile create`")
    } else if config.profiles.len() < config.active_profile {
        println!(
            "{}",
            "Active profile specified incorrectly, please pick a profile to use"
                .red()
                .bold()
        );
        subcommands::profile::switch(config, None)?;
        Ok(&mut config.profiles[config.active_profile])
    } else {
        Ok(&mut config.profiles[config.active_profile])
    }
}

/// Get the active modpack with error handling
fn get_active_modpack(config: &mut Config) -> Result<&mut Modpack> {
    if config.modpacks.is_empty() {
        bail!("There are no modpacks configured, add a modpack using `ferium modpack add`")
    } else if config.modpacks.len() < config.active_modpack {
        println!(
            "{}",
            "Active modpack specified incorrectly, please pick a modpack to use"
                .red()
                .bold()
        );
        subcommands::modpack::switch(config, None)?;
        Ok(&mut config.modpacks[config.active_modpack])
    } else {
        Ok(&mut config.modpacks[config.active_modpack])
    }
}

/// Check if `profile` is empty, and if so return an error
fn check_empty_profile(profile: &Profile) -> Result<()> {
    if profile.mods.is_empty() {
        bail!("Your currently selected profile is empty! Run `ferium help` to see how to add mods");
    }
    Ok(())
}

/// Check for an internet connection
async fn check_internet() -> Result<()> {
    if check(Some(1)).await.is_err() {
        // If it takes more than 1 second
        // show that we're checking the internet connection
        // and check for 4 more seconds
        eprint!("Checking internet connection... ");
        match check(Some(4)).await {
            Ok(_) => {
                println!("{}", *TICK);
                Ok(())
            },
            Err(_) => Err(anyhow!(
                "{} Ferium requires an internet connection to work",
                CROSS
            )),
        }
    } else {
        Ok(())
    }
}
