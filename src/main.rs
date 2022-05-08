mod cli;
mod mutex_ext;
mod subcommands;

use anyhow::{anyhow, bail, Result};
use clap::StructOpt;
use cli::{Ferium, ProfileSubCommands, SubCommands};
use colored::{ColoredString, Colorize};
use ferinth::Ferinth;
use furse::Furse;
use lazy_static::lazy_static;
use libium::config;
use std::sync::Arc;
use subcommands::{add, upgrade};
use tokio::{fs::create_dir_all, io::AsyncReadExt, runtime, spawn};

const CROSS: &str = "×";
lazy_static! {
    pub static ref TICK: ColoredString = "✓".green();
    pub static ref YELLOW_TICK: ColoredString = "✓".yellow();
    pub static ref THEME: dialoguer::theme::ColorfulTheme =
        dialoguer::theme::ColorfulTheme::default();
}

fn main() {
    let cli = Ferium::parse();
    let mut builder = runtime::Builder::new_multi_thread();
    builder.enable_all();
    builder.thread_name("ferium-worker");
    if let Some(threads) = cli.threads {
        builder.max_blocking_threads(threads);
    }
    let runtime = builder.build().expect("Could not initialise Tokio runtime");
    if let Err(err) = runtime.block_on(actual_main(cli)) {
        eprintln!("{}", err.to_string().red().bold());
        runtime.shutdown_background();
        std::process::exit(1);
    }
}

async fn actual_main(cli_app: Ferium) -> Result<()> {
    let github = {
        let mut builder = octocrab::OctocrabBuilder::new();
        if let Some(token) = cli_app.github_token {
            builder = builder.personal_token(token);
        }
        octocrab::initialise(builder)
    }?;
    let modrinth = Arc::new(Ferinth::new());
    let curseforge = Arc::new(Furse::new(env!(
        "CURSEFORGE_API_KEY",
        "A CurseForge API key is required to build. If you don't have one, you can bypass this by setting the variable to a blank string, however anything using the CurseForge API will not work."
    )));
    let mut config_file =
        config::get_file(cli_app.config_file.unwrap_or_else(config::file_path)).await?;
    let mut config_file_contents = String::new();
    config_file
        .read_to_string(&mut config_file_contents)
        .await?;
    let mut config: config::structs::Config = serde_json::from_str(&config_file_contents)?;

    // The create command must run before getting the profile so that configs without profiles can have profiles added to them
    if let SubCommands::Profile {
        subcommand:
            ProfileSubCommands::Create {
                import,
                game_version,
                mod_loader,
                name,
                output_dir,
            },
    } = cli_app.subcommand
    {
        check_internet().await?;
        subcommands::profile::create(
            &mut config,
            import,
            game_version,
            mod_loader,
            name,
            output_dir,
        )
        .await?;

        // Update config file and quit
        config::write_file(&mut config_file, &config).await?;
        return Ok(());
    }

    // Get a mutable reference to the active profile
    let profile = if let Some(profile) = config.profiles.get_mut(config.active_profile) {
        profile
    } else {
        if config.profiles.is_empty() {
            bail!("There are no profiles configured. Add a profile to your config using `ferium profile create`")
        }
        // Default to first profile if index is set incorrectly
        config.active_profile = 0;
        config::write_file(&mut config_file, &config).await?;
        bail!("Active profile specified incorrectly. Switched to first profile",)
    };

    // Run function(s) based on the sub(sub)command to be executed
    match cli_app.subcommand {
        SubCommands::AddModrinth {
            project_id,
            dont_check_game_version,
            dont_check_mod_loader,
        } => {
            check_internet().await?;
            add::modrinth(
                &modrinth,
                &project_id,
                profile,
                Some(!dont_check_game_version),
                Some(!dont_check_mod_loader),
            )
            .await?;
        },
        SubCommands::AddGithub {
            owner,
            name,
            dont_check_game_version,
            dont_check_mod_loader,
        } => {
            check_internet().await?;
            eprint!("Adding mod... ");
            let repo = libium::add::github(
                github.repos(owner, name),
                profile,
                Some(!dont_check_game_version),
                Some(!dont_check_mod_loader),
            )
            .await?;
            println!("{} ({})", *TICK, repo.name);
        },
        SubCommands::AddCurseforge {
            project_id,
            dont_check_game_version,
            dont_check_mod_loader,
        } => {
            check_internet().await?;
            add::curseforge(
                &curseforge,
                project_id,
                profile,
                Some(!dont_check_game_version),
                Some(!dont_check_mod_loader),
            )
            .await?;
        },
        SubCommands::List { verbose } => {
            check_empty_profile(profile)?;
            if verbose {
                check_internet().await?;
                let mut tasks = Vec::new();
                for mod_ in &profile.mods {
                    use config::structs::ModIdentifier;
                    match &mod_.identifier {
                        ModIdentifier::CurseForgeProject(project_id) => tasks.push(spawn(
                            subcommands::list::curseforge(curseforge.clone(), *project_id),
                        )),
                        ModIdentifier::ModrinthProject(project_id) => tasks.push(spawn(
                            subcommands::list::modrinth(modrinth.clone(), project_id.clone()),
                        )),
                        ModIdentifier::GitHubRepository(full_name) => tasks.push(spawn(
                            subcommands::list::github(github.clone(), full_name.clone()),
                        )),
                    };
                }
                for handle in tasks {
                    handle.await??;
                }
            } else {
                for mod_ in &profile.mods {
                    println!("{}", mod_.name);
                }
            }
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
                    profile,
                    game_version,
                    mod_loader,
                    name,
                    output_dir,
                )
                .await?;
            },
            // This must have ran earlier before getting the profile
            ProfileSubCommands::Create { .. } => unreachable!(),
            ProfileSubCommands::Delete { profile_name } => {
                subcommands::profile::delete(&mut config, profile_name)?;
            },
            ProfileSubCommands::List => subcommands::profile::list(&config),
        },
        SubCommands::Remove { mod_names } => {
            check_empty_profile(profile)?;
            subcommands::remove(profile, mod_names)?;
        },
        SubCommands::Switch { profile_name } => subcommands::switch(&mut config, profile_name)?,
        SubCommands::Sort => profile
            .mods
            .sort_by_cached_key(|mod_| mod_.name.to_lowercase()),
        SubCommands::Upgrade => {
            check_internet().await?;
            check_empty_profile(profile)?;
            create_dir_all(&profile.output_dir.join(".old")).await?;
            upgrade(modrinth, curseforge, github, profile).await?;
        },
    };

    // Update config file with possibly edited config
    config::write_file(&mut config_file, &config).await?;

    Ok(())
}

/// Check if `profile` is empty, and if so return an error
fn check_empty_profile(profile: &config::structs::Profile) -> Result<()> {
    if profile.mods.is_empty() {
        bail!("Your currently selected profile is empty! Run `ferium help` to see how to add mods");
    }
    Ok(())
}

/// Check for an internet connection
async fn check_internet() -> Result<()> {
    if online::check(Some(1)).await.is_err() {
        // If it takes more than 1 second
        // show that we're checking the internet connection
        // and check for 4 more seconds
        eprint!("Checking internet connection... ");
        match online::check(Some(4)).await {
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
