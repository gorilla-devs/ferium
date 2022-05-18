mod cli;
mod download;
mod subcommands;

use anyhow::{anyhow, bail, Result};
use clap::StructOpt;
use cli::{Ferium, ModpackSubCommands, ProfileSubCommands, SubCommands};
use colored::{ColoredString, Colorize};
use ferinth::Ferinth;
use furse::Furse;
use indicatif::ProgressStyle;
use lazy_static::lazy_static;
use libium::config::{self, structs::ModIdentifier};
use octocrab::OctocrabBuilder;
use std::sync::Arc;
use subcommands::{add, upgrade};
use tokio::{fs::create_dir_all, runtime, spawn};

const CROSS: &str = "×";
lazy_static! {
    pub static ref TICK: ColoredString = "✓".green();
    pub static ref YELLOW_TICK: ColoredString = "✓".yellow();
    pub static ref THEME: dialoguer::theme::ColorfulTheme =
        dialoguer::theme::ColorfulTheme::default();
    pub static ref STYLE_NO: ProgressStyle = ProgressStyle::default_bar()
        .template("{spinner} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:.cyan}/{len:.blue}")
        .progress_chars("#>-");
    pub static ref STYLE_BYTE: ProgressStyle = ProgressStyle::default_bar()
        .template("{spinner} [{bytes_per_sec}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}",)
        .progress_chars("#>-");
}

fn main() {
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
        runtime.shutdown_background();
        std::process::exit(1);
    }
}

async fn actual_main(cli_app: Ferium) -> Result<()> {
    let github = Arc::new(
        cli_app
            .github_token
            .map_or_else(OctocrabBuilder::new, |token| {
                OctocrabBuilder::new().personal_token(token)
            })
            .build()?,
    );
    let modrinth = Arc::new(Ferinth::new());
    // Yes this is a personal API key, but I am allowed to write it in source.
    // The reason is the API key is used for tracking usage, it's not for authentication.
    // So please don't use this outside of Ferium, although telling you not to is all I can do...
    let curseforge = Arc::new(Furse::new(
        "$2a$10$QbCxI6f4KxEs50QKwE2piu1t6oOA8ayOw27H9N/eaH3Sdp5NTWwvO",
    ));
    let mut config_file =
        config::get_file(cli_app.config_file.unwrap_or_else(config::file_path)).await?;
    let mut config = config::deserialise(&config::read_file(&mut config_file).await?)?;

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

    if let SubCommands::Modpack { subcommand } = &cli_app.subcommand {
        if let ModpackSubCommands::AddCurseforge {
            project_id,
            output_dir,
            install_overrides,
        } = subcommand
        {
            check_internet().await?;
            subcommands::modpack::add::curseforge(
                curseforge.clone(),
                &mut config,
                *project_id,
                output_dir,
                *install_overrides,
            )
            .await?;

            // Update config file and quit
            config::write_file(&mut config_file, &config).await?;
            return Ok(());
        } else if let ModpackSubCommands::AddModrinth {
            project_id,
            output_dir,
            install_overrides,
        } = subcommand
        {
            check_internet().await?;
            subcommands::modpack::add::modrinth(
                modrinth.clone(),
                &mut config,
                project_id,
                output_dir,
                *install_overrides,
            )
            .await?;

            // Update config file and quit
            config::write_file(&mut config_file, &config).await?;
            return Ok(());
        }
        // Get a mutable reference to the active modpack
        let modpack = if let Some(modpack) = config.modpacks.get_mut(config.active_modpack) {
            modpack
        } else {
            if config.modpacks.is_empty() {
                bail!("There are no modpacks configured! Run `ferium modpack help` to see how to add modpacks")
            }
            // Default to first modpack if index is set incorrectly
            config.active_modpack = 0;
            config::write_file(&mut config_file, &config).await?;
            bail!("Active modpack specified incorrectly. Switched to first modpack")
        };

        match subcommand {
            ModpackSubCommands::AddCurseforge { .. } | ModpackSubCommands::AddModrinth { .. } => {
                unreachable!()
            },
            ModpackSubCommands::Configure {
                output_dir,
                install_overrides,
            } => {
                subcommands::modpack::configure(modpack, output_dir, *install_overrides).await?;
            },
            ModpackSubCommands::Delete { modpack_name } => {
                subcommands::modpack::delete(&mut config, modpack_name)?;
            },
            ModpackSubCommands::List => subcommands::modpack::list(&config),
            ModpackSubCommands::Switch { modpack_name } => {
                subcommands::modpack::switch(&mut config, modpack_name)?;
            },
            ModpackSubCommands::Upgrade => {
                check_internet().await?;
                subcommands::modpack::upgrade(modrinth.clone(), curseforge.clone(), modpack)
                    .await?;
            },
        }

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
        bail!("Active profile specified incorrectly. Switched to first profile")
    };

    // Run function(s) based on the sub(sub)command to be executed
    match cli_app.subcommand {
        SubCommands::AddModrinth {
            project_id,
            dont_check_game_version,
            dont_check_mod_loader,
            dont_add_dependencies,
        } => {
            check_internet().await?;
            add::modrinth(
                modrinth,
                &project_id,
                profile,
                Some(!dont_check_game_version),
                Some(!dont_check_mod_loader),
                !dont_add_dependencies,
            )
            .await?;
        },
        SubCommands::AddGithub {
            name,
            dont_check_game_version,
            dont_check_mod_loader,
        } => {
            check_internet().await?;
            let name = name.split('/').collect::<Vec<_>>();
            if name.len() != 2 {
                bail!("Invalid repository name")
            }
            add::github(
                github.repos(name[0], name[1]),
                profile,
                Some(!dont_check_game_version),
                Some(!dont_check_mod_loader),
            )
            .await?;
        },
        SubCommands::AddCurseforge {
            project_id,
            dont_check_game_version,
            dont_check_mod_loader,
            dont_add_dependencies,
        } => {
            check_internet().await?;
            add::curseforge(
                curseforge,
                project_id,
                profile,
                Some(!dont_check_game_version),
                Some(!dont_check_mod_loader),
                !dont_add_dependencies,
            )
            .await?;
        },
        SubCommands::List { verbose } => {
            check_empty_profile(profile)?;
            if verbose {
                check_internet().await?;
                let mut tasks = Vec::new();
                for mod_ in &profile.mods {
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
        SubCommands::Modpack { .. } => unreachable!(),
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
            ProfileSubCommands::Switch { profile_name } => {
                subcommands::profile::switch(&mut config, profile_name)?;
            },
        },
        SubCommands::Remove { mod_names } => {
            check_empty_profile(profile)?;
            subcommands::remove(profile, mod_names)?;
        },
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
