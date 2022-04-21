mod cli;
mod subcommands;

use anyhow::{bail, Result};
use clap::StructOpt;
use cli::{Ferium, ProfileSubCommands, SubCommands};
use colored::{ColoredString, Colorize};
use ferinth::Ferinth;
use furse::Furse;
use itertools::Itertools;
use lazy_static::lazy_static;
use libium::{add, config, upgrade};
use tokio::{
    fs::{create_dir_all, remove_file},
    io::AsyncReadExt,
};

const CROSS: &str = "×";
lazy_static! {
    pub static ref TICK: ColoredString = "✓".green();
}

struct Downloadable {
    filename: String,
    download_url: String,
}
impl From<furse::structures::file_structs::File> for Downloadable {
    fn from(file: furse::structures::file_structs::File) -> Self {
        Self {
            filename: file.file_name,
            download_url: file.download_url,
        }
    }
}
impl From<ferinth::structures::version_structs::VersionFile> for Downloadable {
    fn from(file: ferinth::structures::version_structs::VersionFile) -> Self {
        Self {
            filename: file.filename,
            download_url: file.url,
        }
    }
}
impl From<octocrab::models::repos::Asset> for Downloadable {
    fn from(asset: octocrab::models::repos::Asset) -> Self {
        Self {
            filename: asset.name,
            download_url: asset.url.into(),
        }
    }
}

#[tokio::main]
async fn main() {
    if let Err(err) = actual_main().await {
        eprintln!("{}", err.to_string().red().bold());
        std::process::exit(1);
    }
}

async fn actual_main() -> Result<()> {
    // This also displays the help page or version automatically
    let cli_app = Ferium::parse();

    // Check for an internet connection
    if online::check(Some(1)).await.is_err() {
        // If it takes more than 1 second
        // show that we're checking the internet connection
        // and check for 4 more seconds
        eprint!("Checking internet connection... ");
        match online::check(Some(4)).await {
            Ok(_) => println!("{}", *TICK),
            Err(_) => bail!("{} Ferium requires an internet connection to work", CROSS),
        }
    };

    let github = octocrab::instance();
    let modrinth = Ferinth::new();
    let curseforge = Furse::new(env!(
        "CURSEFORGE_API_KEY",
        "A CurseForge API key is required to build. If you don't have one, you can bypass this by setting the variable to a blank string, however anything using the CurseForge API will not work."
    ));
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
                game_version,
                force_game_version,
                mod_loader,
                name,
                output_dir,
            },
    } = cli_app.subcommand
    {
        subcommands::profile::create(
            &modrinth,
            &mut config,
            game_version,
            force_game_version,
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
        SubCommands::AddModrinth { project_id } => {
            eprint!("Adding mod... ");
            let project = add::modrinth(&modrinth, project_id, profile).await?;
            println!("{} ({})", *TICK, project.title);
        },
        SubCommands::AddGithub { owner, name } => {
            eprint!("Adding mod... ");
            let repo = add::github(github.repos(owner, name), profile).await?;
            println!("{} ({})", *TICK, repo.name);
        },
        SubCommands::AddCurseforge { project_id } => {
            eprint!("Adding mod... ");
            let project = add::curseforge(&curseforge, project_id, profile).await?;
            println!("{} ({})", *TICK, project.name);
        },
        SubCommands::List { verbose } => {
            check_empty_profile(profile)?;
            for mod_ in &profile.mods {
                if verbose {
                    use config::structs::ModIdentifier;
                    match &mod_.identifier {
                        ModIdentifier::CurseForgeProject(project_id) => {
                            subcommands::list::curseforge(&curseforge, *project_id).await
                        },
                        ModIdentifier::ModrinthProject(project_id) => {
                            subcommands::list::modrinth(&modrinth, project_id).await
                        },
                        ModIdentifier::GitHubRepository(full_name) => {
                            subcommands::list::github(&github, full_name).await
                        },
                    }?;
                } else {
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
        SubCommands::Switch { profile_name } => {
            subcommands::switch(&mut config, profile_name)?;
        },
        SubCommands::Sort => profile.mods.sort_by_cached_key(|mod_| mod_.name.clone()),
        SubCommands::Upgrade => {
            check_empty_profile(profile)?;
            create_dir_all(&profile.output_dir).await?;
            let mut to_download = Vec::new();
            let mut error = false;

            println!("{}\n", "Determining the Latest Compatible Versions".bold());
            for mod_ in &profile.mods {
                use libium::config::structs::ModIdentifier;
                let result: Result<Downloadable, _> = match &mod_.identifier {
                    ModIdentifier::CurseForgeProject(project_id) => upgrade::curseforge(
                        &curseforge,
                        profile,
                        *project_id,
                        mod_.check_game_version,
                        mod_.check_mod_loader,
                    )
                    .await
                    .map(std::convert::Into::into),
                    ModIdentifier::ModrinthProject(project_id) => upgrade::modrinth(
                        &modrinth,
                        profile,
                        project_id,
                        mod_.check_game_version,
                        mod_.check_mod_loader,
                    )
                    .await
                    .map(|version| {
                        for file in &version.files {
                            if file.primary {
                                return file.clone().into();
                            }
                        }
                        version.files[0].clone().into()
                    }),
                    ModIdentifier::GitHubRepository(full_name) => upgrade::github(
                        &github.repos(&full_name.0, &full_name.1),
                        profile,
                        mod_.check_game_version,
                        mod_.check_mod_loader,
                    )
                    .await
                    .map(std::convert::Into::into),
                };
                match result {
                    Ok(result) => {
                        println!(
                            "{} {:40}{}",
                            *TICK,
                            mod_.name,
                            format!("({})", result.filename).dimmed()
                        );
                        to_download.push(result);
                    },
                    Err(err) => {
                        eprintln!("{}", format!("{} {:40}{}", CROSS, mod_.name, err).red());
                        error = true;
                    },
                }
            }

            eprint!("\n{}", "Downloading Mod Files... ".bold());
            for file in std::fs::read_dir(&profile.output_dir)? {
                let file = file?;
                let path = file.path();
                if path.is_file() {
                    let mut index = None;
                    // If a file is already downloaded
                    if let Some(downloadable) = to_download
                        .iter()
                        .find_position(|thing| file.file_name().to_str().unwrap() == thing.filename)
                    {
                        index = Some(downloadable.0);
                    }
                    match index {
                        // Then don't download the file
                        Some(index) => {
                            to_download.swap_remove(index);
                        },
                        // Or else delete the file
                        None => remove_file(path).await?,
                    }
                }
            }
            match {
                for downloadable in to_download {
                    let contents = reqwest::get(downloadable.download_url)
                        .await?
                        .bytes()
                        .await?;
                    upgrade::write_mod_file(profile, contents, &downloadable.filename).await?;
                }
                Ok::<(), anyhow::Error>(())
            } {
                Ok(_) => println!("{}", *TICK),
                Err(_) => bail!("{}", CROSS),
            }

            if error {
                bail!("\nCould not get the latest compatible version of some mods")
            }
        },
    };

    // Update config file with new values
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
