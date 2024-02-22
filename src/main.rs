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
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_truncation,
    clippy::multiple_crate_versions,
    clippy::large_enum_variant,
    clippy::too_many_lines
)]

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
use itertools::Itertools;
use libium::{
    config::{
        self,
        structs::{Config, ModIdentifier, Modpack, Profile},
        DEFAULT_CONFIG_PATH,
    },
    read_wrapper,
};
use octocrab::OctocrabBuilder;
use once_cell::sync::Lazy;
use std::{
    env::{var, var_os},
    process::ExitCode,
};
use tokio::runtime;

const CROSS: &str = "×";
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
    #[cfg(windows)]
    {
        #[allow(clippy::unwrap_used)] // There is actually no error
        // Enable colours on conhost
        colored::control::set_virtual_terminal(true).unwrap();
    }
    #[allow(clippy::expect_used)] // No error handling yet
    let runtime = builder.build().expect("Could not initialise Tokio runtime");
    if let Err(err) = runtime.block_on(actual_main(cli)) {
        eprintln!("{}", err.to_string().red().bold());
        if err
            .to_string()
            .to_lowercase()
            .contains("error trying to connect")
        {
            eprintln!(
                "{}",
                "Verify that you are connnected to the internet"
                    .yellow()
                    .bold()
            );
        }
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

async fn actual_main(mut cli_app: Ferium) -> Result<()> {
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
    // Alias `ferium profiles` to `ferium profile list`
    if let SubCommands::Profiles = cli_app.subcommand {
        cli_app.subcommand = SubCommands::Profile {
            subcommand: Some(ProfileSubCommands::List),
        };
    }
    // Alias `ferium modpacks` to `ferium modpack list`
    if let SubCommands::Modpacks = cli_app.subcommand {
        cli_app.subcommand = SubCommands::Modpack {
            subcommand: Some(ModpackSubCommands::List),
        };
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
        &cli_app
            .config_file
            .or_else(|| var_os("FERIUM_CONFIG_FILE").map(Into::into))
            .unwrap_or(DEFAULT_CONFIG_PATH.clone()),
    )
    .await?;
    let mut config = config::deserialise(&read_wrapper(&mut config_file).await?)?;

    // Run function(s) based on the sub(sub)command to be executed
    match cli_app.subcommand {
        SubCommands::Complete { .. } | SubCommands::Profiles | SubCommands::Modpacks => {
            unreachable!();
        }
        SubCommands::Add {
            identifier,
            force,
            ignore_game_version,
            ignore_mod_loader,
        } => {
            let profile = get_active_profile(&mut config)?;
            eprint!("Adding mod... ");
            if let Ok(project_id) = identifier.parse() {
                let name = libium::add::curseforge(
                    &curseforge,
                    project_id,
                    profile,
                    !force,
                    !ignore_game_version,
                    !ignore_mod_loader,
                )
                .await?;
                println!("{} {}", *TICK, name.bold());
            } else if identifier.split('/').count() == 2 {
                let split = identifier.split('/').collect::<Vec<_>>();
                let name = libium::add::github(
                    &github.build()?.repos(split[0], split[1]),
                    profile,
                    !force,
                    !ignore_game_version,
                    !ignore_mod_loader,
                )
                .await?;
                println!("{} {}", *TICK, name.bold());
            } else {
                match libium::add::modrinth(
                    &modrinth,
                    &identifier,
                    profile,
                    !force,
                    !ignore_game_version,
                    !ignore_mod_loader,
                )
                .await
                {
                    Ok((name, donation_urls)) => {
                        println!("{} {}", *TICK, name.bold());

                        if !donation_urls.is_empty() {
                            println!(
                                "Consider supporting the mod creator on {}",
                                donation_urls
                                    .iter()
                                    .map(|this| format!(
                                        "{} ({})",
                                        this.platform.bold(),
                                        this.url.to_string().blue().underline()
                                    ))
                                    .format(" or ")
                            );
                        }
                    }
                    Err(err) => {
                        if err.to_string() == ferinth::Error::InvalidIDorSlug.to_string() {
                            bail!("Invalid identifier")
                        }
                        bail!(err)
                    }
                }
            }
        }
        SubCommands::List { verbose, markdown } => {
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile)?;
            if verbose {
                subcommands::list::verbose(modrinth, curseforge, profile, markdown).await?;
            } else {
                println!(
                    "{} {} on {} {}\n",
                    profile.name.bold(),
                    format!("({} mods)", profile.mods.len()).yellow(),
                    format!("{:?}", profile.mod_loader).purple(),
                    profile.game_version.green(),
                );
                for mod_ in &profile.mods {
                    println!(
                        "{:20}  {}",
                        match &mod_.identifier {
                            ModIdentifier::CurseForgeProject(id) =>
                                format!("{} {:8}", "CF".red(), id.to_string().dimmed()),
                            ModIdentifier::ModrinthProject(id) =>
                                format!("{} {:8}", "MR".green(), id.dimmed()),
                            ModIdentifier::GitHubRepository(_) => "GH".purple().to_string(),
                        },
                        match &mod_.identifier {
                            ModIdentifier::ModrinthProject(_)
                            | ModIdentifier::CurseForgeProject(_) => mod_.name.bold().to_string(),
                            ModIdentifier::GitHubRepository(id) =>
                                format!("{}/{}", id.0.dimmed(), id.1.bold()),
                        },
                    );
                }
            }
        }
        SubCommands::Modpack { subcommand } => {
            let mut default_flag = false;
            let subcommand = subcommand.unwrap_or_else(|| {
                default_flag = true;
                ModpackSubCommands::Info
            });
            match subcommand {
                ModpackSubCommands::Add {
                    identifier,
                    output_dir,
                    install_overrides,
                } => {
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
                                anyhow!("Invalid identifier")
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
                ModpackSubCommands::Delete {
                    modpack_name,
                    switch_to,
                } => {
                    subcommands::modpack::delete(&mut config, modpack_name, switch_to)?;
                }
                ModpackSubCommands::Info => {
                    subcommands::modpack::info(get_active_modpack(&mut config)?, true);
                }
                ModpackSubCommands::List => {
                    for (i, modpack) in config.modpacks.iter().enumerate() {
                        subcommands::modpack::info(modpack, i == config.active_modpack);
                    }
                }
                ModpackSubCommands::Switch { modpack_name } => {
                    subcommands::modpack::switch(&mut config, modpack_name)?;
                }
                ModpackSubCommands::Upgrade => {
                    subcommands::modpack::upgrade(
                        &modrinth,
                        &curseforge,
                        get_active_modpack(&mut config)?,
                    )
                    .await?;
                }
            };
            if default_flag {
                println!(
                    "{} ferium modpack help {}",
                    "Use".yellow(),
                    "for more information about this subcommand".yellow()
                );
            }
        }
        SubCommands::Profile { subcommand } => {
            let mut default_flag = false;
            let subcommand = subcommand.unwrap_or_else(|| {
                default_flag = true;
                ProfileSubCommands::Info
            });
            match subcommand {
                ProfileSubCommands::Configure {
                    game_version,
                    mod_loader,
                    name,
                    output_dir,
                } => {
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
                ProfileSubCommands::Delete {
                    profile_name,
                    switch_to,
                } => {
                    subcommands::profile::delete(&mut config, profile_name, switch_to)?;
                }
                ProfileSubCommands::Info => {
                    subcommands::profile::info(get_active_profile(&mut config)?, true);
                }

                ProfileSubCommands::List => {
                    for (i, profile) in config.profiles.iter().enumerate() {
                        subcommands::profile::info(profile, i == config.active_profile);
                    }
                }

                ProfileSubCommands::Switch { profile_name } => {
                    subcommands::profile::switch(&mut config, profile_name)?;
                }
            };
            if default_flag {
                println!(
                    "{} ferium profile help {}",
                    "Use".yellow(),
                    "for more information about this subcommand".yellow()
                );
            }
        }
        SubCommands::Remove { mod_names } => {
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile)?;
            subcommands::remove(profile, mod_names)?;
        }
        SubCommands::Upgrade => {
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
