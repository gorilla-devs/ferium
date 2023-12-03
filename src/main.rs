#![deny(
    clippy::all,
    clippy::perf,
    clippy::cargo,
    clippy::style,
    clippy::pedantic,
    clippy::suspicious,
    clippy::complexity,
    clippy::create_dir,
    clippy::unwrap_used,
    clippy::expect_used, // use anyhow::Context instead
    clippy::correctness
)]
#![warn(clippy::dbg_macro, clippy::expect_used)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::multiple_crate_versions,
    clippy::large_enum_variant,
    clippy::too_many_lines
)]

mod cli;
mod download;
mod subcommands;

use anyhow::{anyhow, bail, Context, Result};
use clap::{CommandFactory, Parser};
use cli::{Ferium, ModpackSubCommands, ProfileSubCommands, SubCommands};
use colored::{ColoredString, Colorize};
use dialoguer::theme::ColorfulTheme;
use ferinth::Ferinth;
use furse::Furse;
use indicatif::ProgressStyle;
use libium::config::{
    self,
    structs::{Config, ModIdentifier, Modpack, Profile},
};
use octocrab::OctocrabBuilder;
use once_cell::sync::Lazy;
use std::{
    env::{var, var_os},
    process::ExitCode,
};
use tokio::{runtime, task::JoinSet};

const CROSSSIGN: &str = "×";
pub static TICK: Lazy<ColoredString> = Lazy::new(|| "✓".green());
pub static YELLOW_TICK: Lazy<ColoredString> = Lazy::new(|| "✓".yellow());
pub static THEME: Lazy<ColorfulTheme> = Lazy::new(Default::default);
#[allow(clippy::expect_used)]
pub static STYLE_NO: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::default_bar()
        .template("{spinner} {elapsed} [{wide_bar:.cyan/blue}] {pos:.cyan}/{len:.blue}")
        .expect("Progress bar template parse failure")
        .progress_chars("#>-")
});
#[allow(clippy::expect_used)]
pub static STYLE_BYTE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::default_bar()
        .template(
            "{spinner} {bytes_per_sec} [{wide_bar:.cyan/blue}] {bytes:.cyan}/{total_bytes:.blue}",
        )
        .expect("Progress bar template parse failure")
        .progress_chars("#>-")
});

fn main() -> ExitCode {
    let cli = Ferium::parse();
    let mut builder = runtime::Builder::new_multi_thread();
    builder.enable_all();
    builder.thread_name("ferium-worker");
    if let Some(threads) = cli.threads {
        builder.worker_threads(threads);
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

    let mut github = OctocrabBuilder::new();
    if let Some(token) = cli_app.github_token {
        github = github.personal_token(token);
    } else if let Ok(token) = var("GITHUB_TOKEN") {
        github = github.personal_token(token);
    }

    let modrinth = Ferinth::new(
        "ferium",
        option_env!("CARGO_PKG_VERSION"),
        Some("Discord: therookiecoder"),
        None,
    )?;

    let curseforge = Furse::new(&cli_app.curseforge_api_key.unwrap_or_else(|| {
        var("CURSEFORGE_API_KEY").unwrap_or_else(|_| {
            "$2a$10$sI.yRk4h4R49XYF94IIijOrO4i3W3dAFZ4ssOlNE10GYrDhc2j8K.".into()
        })
    }));

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
            dependencies
        } => {
            let profile = get_active_profile(&mut config)?;
            check_internet().await?;
            if let Ok(project_id) = identifier.parse::<i32>() {
                subcommands::add::curseforge(
                    &curseforge,
                    project_id,
                    profile,
                    Some(!dont_check_game_version),
                    Some(!dont_check_mod_loader),
                    dependencies
                )
                .await?;
            } else if identifier.split('/').count() == 2 {
                let split = identifier.split('/').collect::<Vec<_>>();
                subcommands::add::github(
                    github.build()?.repos(split[0], split[1]),
                    profile,
                    Some(!dont_check_game_version),
                    Some(!dont_check_mod_loader),
                )
                .await?;
            } else if let Err(err) = subcommands::add::modrinth(
                &modrinth,
                &identifier,
                profile,
                Some(!dont_check_game_version),
                Some(!dont_check_mod_loader),
                // dependencies,
            )
            .await
            {
                return Err(
                    if err.to_string() == ferinth::Error::InvalidIDorSlug.to_string() {
                        anyhow!("Invalid indentifier")
                    } else {
                        err
                    },
                );
            }
        }
        SubCommands::List { verbose, markdown } => {
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile)?;
            if verbose {
                check_internet().await?;
                let github = github.build()?;
                let mut tasks = JoinSet::new();
                let mut mr_ids = Vec::<&str>::new();
                for mod_ in &profile.mods {
                    if markdown {
                        match &mod_.identifier {
                            ModIdentifier::CurseForgeProject(project_id) => {
                                subcommands::list::curseforge_md(&curseforge, *project_id).await?;
                            }
                            ModIdentifier::ModrinthProject(project_id) => {
                                subcommands::list::modrinth_md(&modrinth, project_id.clone())
                                    .await?;
                            }
                            ModIdentifier::GitHubRepository(full_name) => {
                                subcommands::list::github_md(&github, full_name.clone()).await?;
                            }
                        };
                    } else {
                        match &mod_.identifier {
                            ModIdentifier::CurseForgeProject(project_id) => {
                                tasks.spawn(subcommands::list::curseforge(
                                    curseforge.clone(),
                                    *project_id,
                                ));
                            }
                            ModIdentifier::ModrinthProject(project_id) => mr_ids.push(project_id),
                            ModIdentifier::GitHubRepository(full_name) => {
                                tasks.spawn(subcommands::list::github(
                                    github.clone(),
                                    full_name.clone(),
                                ));
                            }
                        };
                    }
                }

                eprint!("Querying Modrinth mod metadata... ");
                let mr_projects = modrinth.get_multiple_projects(&mr_ids).await?;
                let mr_teams_members = modrinth
                    .list_multiple_teams_members(
                        &mr_projects
                            .iter()
                            .map(|p| &p.team as &str)
                            .collect::<Vec<_>>(),
                    )
                    .await?;
                println!("{}", &*TICK);
                for (project, team_members) in
                    mr_projects.into_iter().zip(mr_teams_members.into_iter())
                {
                    tasks.spawn(
                        async move { Ok(subcommands::list::modrinth(project, team_members)) },
                    );
                }

                while let Some(res) = tasks.join_next().await {
                    let (id, name) = res??;
                    profile
                        .mods
                        .iter_mut()
                        .find(|mod_| mod_.identifier == id)
                        .context("Could not find expected mod")?
                        .name = name;
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
        }
        SubCommands::Modpack { subcommand } => match subcommand {
            ModpackSubCommands::Add {
                identifier,
                output_dir,
                install_overrides,
            } => {
                check_internet().await?;
                if let Ok(project_id) = identifier.parse::<i32>() {
                    subcommands::modpack::add::curseforge(
                        &curseforge,
                        &mut config,
                        project_id,
                        output_dir,
                        install_overrides,
                    )
                    .await?;
                } else if let Err(err) = subcommands::modpack::add::modrinth(
                    &modrinth,
                    &mut config,
                    &identifier,
                    output_dir,
                    install_overrides,
                )
                .await
                {
                    return Err(
                        if err.to_string() == ferinth::Error::InvalidIDorSlug.to_string() {
                            anyhow!("Invalid indentifier")
                        } else {
                            err
                        },
                    );
                }
            }
            ModpackSubCommands::Configure {
                output_dir,
                install_overrides,
            } => {
                subcommands::modpack::configure(
                    get_active_modpack(&mut config)?,
                    output_dir,
                    install_overrides,
                )?;
            }
            ModpackSubCommands::Delete { modpack_name } => {
                subcommands::modpack::delete(&mut config, modpack_name)?;
            }
            ModpackSubCommands::List => {
                if config.modpacks.is_empty() {
                    bail!("There are no modpacks configured, add a modpack using `ferium modpack add`")
                }
                subcommands::modpack::list(&config);
            }
            ModpackSubCommands::Switch { modpack_name } => {
                subcommands::modpack::switch(&mut config, modpack_name)?;
            }
            ModpackSubCommands::Upgrade => {
                check_internet().await?;
                subcommands::modpack::upgrade(
                    &modrinth,
                    &curseforge,
                    get_active_modpack(&mut config)?,
                )
                .await?;
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
                    get_active_profile(&mut config)?,
                    game_version,
                    mod_loader,
                    name,
                    output_dir,
                )
                .await?;
            }
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
            }
            ProfileSubCommands::Delete { profile_name } => {
                subcommands::profile::delete(&mut config, profile_name)?;
            }
            ProfileSubCommands::List => {
                if config.profiles.is_empty() {
                    bail!("There are no profiles configured, create a profile using `ferium profile create`")
                }
                subcommands::profile::list(&config);
            }
            ProfileSubCommands::Switch { profile_name } => {
                subcommands::profile::switch(&mut config, profile_name)?;
            }
        },
        SubCommands::Remove { mod_names } => {
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile)?;
            subcommands::remove(profile, mod_names)?;
        }
        SubCommands::Upgrade => {
            check_internet().await?;
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile)?;
            subcommands::upgrade(modrinth, curseforge, github.build()?, profile).await?;
        }
    };

    config.profiles.iter_mut().for_each(|profile| {
        profile
            .mods
            .sort_unstable_by_key(|mod_| mod_.name.to_lowercase());
    });
    // Update config file with possibly edited config
    config::write_file(&mut config_file, &config).await?;

    Ok(())
}

/// Get the active profile with error handling
fn get_active_profile(config: &mut Config) -> Result<&mut Profile> {
    match config.profiles.len() {
        0 => {
            bail!("There are no profiles configured, add a profile using `ferium profile create`")
        }
        1 => config.active_profile = 0,
        n if n <= config.active_profile => {
            println!(
                "{}",
                "Active profile specified incorrectly, please pick a profile to use"
                    .red()
                    .bold()
            );
            subcommands::profile::switch(config, None)?;
        }
        _ => (),
    }
    Ok(&mut config.profiles[config.active_profile])
}

/// Get the active modpack with error handling
fn get_active_modpack(config: &mut Config) -> Result<&mut Modpack> {
    match config.modpacks.len() {
        0 => bail!("There are no modpacks configured, add a modpack using `ferium modpack add`"),
        1 => config.active_modpack = 0,
        n if n <= config.active_modpack => {
            println!(
                "{}",
                "Active modpack specified incorrectly, please pick a modpack to use"
                    .red()
                    .bold()
            );
            subcommands::modpack::switch(config, None)?;
        }
        _ => (),
    }
    Ok(&mut config.modpacks[config.active_modpack])
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
    let client = reqwest::Client::default();
    client
        .get(ferinth::BASE_URL.as_ref())
        .send()
        .await?
        .error_for_status()?;
    client
        .get("https://api.curseforge.com/")
        .send()
        .await?
        .error_for_status()?;
    client.get("https://api.github.com/").send().await?;

    Ok(())
}
