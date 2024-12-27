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
    clippy::correctness,
    clippy::allow_attributes,
)]
#![warn(clippy::dbg_macro)]
#![expect(clippy::multiple_crate_versions, clippy::too_many_lines)]

mod add;
mod cli;
mod download;
mod file_picker;
mod subcommands;

#[cfg(test)]
mod tests;

use anyhow::{anyhow, bail, ensure, Result};
use clap::{CommandFactory, Parser};
use cli::{Ferium, ModpackSubCommands, ProfileSubCommands, SubCommands};
use colored::{ColoredString, Colorize};
use indicatif::ProgressStyle;
use libium::{
    config::{
        self,
        filters::ProfileParameters as _,
        structs::{Config, ModIdentifier, Modpack, Profile},
        DEFAULT_CONFIG_PATH,
    },
    iter_ext::IterExt as _,
};
use std::{
    env::{set_var, var_os},
    process::ExitCode,
    sync::{LazyLock, OnceLock},
};

const CROSS: &str = "×";
static TICK: LazyLock<ColoredString> = LazyLock::new(|| "✓".green());

pub static SEMAPHORE: OnceLock<tokio::sync::Semaphore> = OnceLock::new();
pub const DEFAULT_PARALLEL_NETWORK: usize = 50;

/// Indicatif themes
#[expect(clippy::expect_used)]
pub static STYLE_NO: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::default_bar()
        .template("{spinner} {elapsed} [{wide_bar:.cyan/blue}] {pos:.cyan}/{len:.blue}")
        .expect("Progress bar template parse failure")
        .progress_chars("#>-")
});
#[expect(clippy::expect_used)]
pub static STYLE_BYTE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::default_bar()
        .template(
            "{spinner} {bytes_per_sec} [{wide_bar:.cyan/blue}] {bytes:.cyan}/{total_bytes:.blue}",
        )
        .expect("Progress bar template parse failure")
        .progress_chars("#>-")
});

fn main() -> ExitCode {
    #[cfg(windows)]
    // Enable colours on conhost (command prompt or powershell)
    {
        #[expect(clippy::unwrap_used, reason = "There is actually no error")]
        colored::control::set_virtual_terminal(true).unwrap();
    }

    let cli = Ferium::parse();

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    builder.thread_name("ferium-worker");
    if let Some(threads) = cli.threads {
        builder.worker_threads(threads);
    }
    #[expect(clippy::expect_used)] // No error handling yet
    let runtime = builder.build().expect("Could not initialise Tokio runtime");

    if let Err(err) = runtime.block_on(actual_main(cli)) {
        if !err.to_string().is_empty() {
            eprintln!("{}", err.to_string().red().bold());
            if err
                .to_string()
                .to_lowercase()
                .contains("error trying to connect")
                || err
                    .to_string()
                    .to_lowercase()
                    .contains("error sending request")
            {
                eprintln!(
                    "{}",
                    "Verify that you are connnected to the internet"
                        .yellow()
                        .bold()
                );
            }
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

    if let Some(token) = cli_app.github_token {
        set_var("GITHUB_TOKEN", token);
    }
    if let Some(key) = cli_app.curseforge_api_key {
        set_var("CURSEFORGE_API_KEY", key);
    }
    let _ = SEMAPHORE.set(tokio::sync::Semaphore::new(cli_app.parallel_network));

    let config_path = &cli_app
        .config_file
        .or_else(|| var_os("FERIUM_CONFIG_FILE").map(Into::into))
        .unwrap_or(DEFAULT_CONFIG_PATH.clone());
    let mut config = config::read_config(config_path)?;

    let mut did_add_fail = false;

    // Run function(s) based on the sub(sub)command to be executed
    match cli_app.subcommand {
        SubCommands::Complete { .. } | SubCommands::Profiles | SubCommands::Modpacks => {
            unreachable!();
        }
        SubCommands::Scan {
            platform,
            directory,
            force,
        } => {
            let profile = get_active_profile(&mut config)?;

            let spinner = indicatif::ProgressBar::new_spinner().with_message("Reading files");
            spinner.enable_steady_tick(std::time::Duration::from_millis(100));

            let ids = libium::scan(directory.as_ref().unwrap_or(&profile.output_dir), || {
                spinner.set_message("Querying servers");
            })
            .await?;

            spinner.set_message("Adding mods");

            let mut send_ids = Vec::new();
            for id in ids {
                use libium::config::structs::ModIdentifier;
                match id {
                    (filename, None, None) => {
                        println!("{} {}", "Unknown file:".yellow(), filename.dimmed());
                    }
                    (_, Some(mr_id), None) => {
                        send_ids.push(ModIdentifier::ModrinthProject(mr_id));
                    }
                    (_, None, Some(cf_id)) => {
                        send_ids.push(ModIdentifier::CurseForgeProject(cf_id));
                    }
                    (_, Some(mr_id), Some(cf_id)) => match platform {
                        cli::Platform::Modrinth => {
                            send_ids.push(ModIdentifier::ModrinthProject(mr_id));
                        }
                        cli::Platform::Curseforge => {
                            send_ids.push(ModIdentifier::CurseForgeProject(cf_id));
                        }
                    },
                }
            }

            let (successes, failures) =
                libium::add(profile, send_ids, !force, false, vec![]).await?;
            spinner.finish_and_clear();

            did_add_fail = add::display_successes_failures(&successes, failures);
        }
        SubCommands::Add {
            identifiers,
            force,
            filters,
        } => {
            let profile = get_active_profile(&mut config)?;
            let override_profile = filters.override_profile;
            let filters: Vec<_> = filters.into();

            if identifiers.len() > 1 && !filters.is_empty() {
                bail!("Only configure filters when adding a single mod!")
            }

            let (successes, failures) = libium::add(
                profile,
                identifiers
                    .into_iter()
                    .map(libium::add::parse_id)
                    .collect_vec(),
                !force,
                override_profile,
                filters,
            )
            .await?;

            did_add_fail = add::display_successes_failures(&successes, failures);
        }
        SubCommands::List { verbose, markdown } => {
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile)?;

            if verbose {
                subcommands::list::verbose(profile, markdown).await?;
            } else {
                println!(
                    "{} {} on {} {}\n",
                    profile.name.bold(),
                    format!("({} mods)", profile.mods.len()).yellow(),
                    profile
                        .filters
                        .mod_loader()
                        .map(ToString::to_string)
                        .unwrap_or_default()
                        .purple(),
                    profile
                        .filters
                        .game_versions()
                        .unwrap_or(&vec![])
                        .iter()
                        .display(", ")
                        .green(),
                );
                for mod_ in &profile.mods {
                    println!(
                        "{:20}  {}",
                        match &mod_.identifier {
                            ModIdentifier::CurseForgeProject(id) =>
                                format!("{} {:8}", "CF".red(), id.to_string().dimmed()),
                            ModIdentifier::ModrinthProject(id) =>
                                format!("{} {:8}", "MR".green(), id.dimmed()),
                            ModIdentifier::GitHubRepository(..) => "GH".purple().to_string(),
                            _ => todo!(),
                        },
                        match &mod_.identifier {
                            ModIdentifier::ModrinthProject(_)
                            | ModIdentifier::CurseForgeProject(_) => mod_.name.bold().to_string(),
                            ModIdentifier::GitHubRepository(owner, repo) =>
                                format!("{}/{}", owner.dimmed(), repo.bold()),
                            _ => todo!(),
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
                            &mut config,
                            project_id,
                            output_dir,
                            install_overrides,
                        )
                        .await?;
                    } else if let Err(err) = subcommands::modpack::add::modrinth(
                        &mut config,
                        &identifier,
                        output_dir,
                        install_overrides,
                    )
                    .await
                    {
                        return Err(
                            if let Some(&ferinth::Error::InvalidIDorSlug) = err.downcast_ref() {
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
                    subcommands::modpack::upgrade(get_active_modpack(&mut config)?).await?;
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
                    game_versions,
                    mod_loaders,
                    name,
                    output_dir,
                } => {
                    subcommands::profile::configure(
                        get_active_profile(&mut config)?,
                        game_versions,
                        mod_loaders,
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
                        if game_version.is_empty() {
                            None
                        } else {
                            Some(game_version)
                        },
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
            subcommands::upgrade(profile).await?;
        }
    };

    config.profiles.iter_mut().for_each(|profile| {
        profile
            .mods
            .sort_unstable_by_key(|mod_| mod_.name.to_lowercase());
    });
    // Update config file with possibly edited config
    config::write_config(config_path, &config)?;

    if did_add_fail {
        Err(anyhow!(""))
    } else {
        Ok(())
    }
}

/// Get the active profile with error handling
fn get_active_profile(config: &mut Config) -> Result<&mut Profile> {
    match config.profiles.len() {
        0 => {
            bail!("There are no profiles configured, add a profile using `ferium profile create`")
        }
        1 => config.active_profile = 0,
        n if config.active_profile >= n => {
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
    ensure!(
        !profile.mods.is_empty(),
        "Your currently selected profile is empty! Run `ferium help` to see how to add mods"
    );
    Ok(())
}
