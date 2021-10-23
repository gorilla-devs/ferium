mod labrinth;
mod octorok;
mod util;

use dialoguer::MultiSelect;
use labrinth::calls::*;
use octorok::calls::*;
use reqwest::Client;
use std::{
    fs::{File, OpenOptions},
    io::Write,
};
use util::{
    cli::SubCommand,
    ferium_error::{FError, FResult},
    *,
};

#[tokio::main]
async fn main() {
    // Error catching and error messages
    match actual_main().await {
        Ok(_) => (),
        Err(err) => match err {
            FError::EmptyConfigFile => println!("× Your config file is empty! Run `ferium help` to see how to add mods or repositories"),
            FError::HTTPError { message } => println!("× An HTTP(S) request returned an error, {}", message),
            FError::InvalidDeviceError => println!("× The device you are currently running on is unsupported by Ferium"),
            FError::IOError {description} => println!("× Encountered an Input/Output error, {}", description),
            FError::JsonError { category } => match category {
                serde_json::error::Category::Syntax => {
                    println!("× Syntax error encountered in JSON file")
                },
                serde_json::error::Category::Data => {
                    println!("× Non matching type while deserialising JSON")
                },
                serde_json::error::Category::Eof => {
                    println!("× Unexpected end of file while reading JSON")
                },
                serde_json::error::Category::Io => {
                    println!("× Encountered an Input/Output error while handling JSON")
                },
            },
            FError::NativeDialogError => println!("× An error occured while interacting with native device libraries"),
            FError::OptionError => println!("× Could not access an expected value"),
            FError::Quit { message } => println!("× {}", message),
            FError::RegexError => println!("× Failed to parse regular expression"),
            FError::ReqwestError { error }=> println!("× Failed to send/process an HTTP(S) request due to {}", error),
        },
    };
}

async fn actual_main() -> FResult<()> {
    // Check for an internet connection
    match online::check(Some(3)).await {
        Ok(_) => (),
        Err(_) => {
            wrappers::print("Checking internet connection... ");
            match online::check(Some(7)).await {
                Ok(_) => println!("✓"),
                Err(_) => {
                    return Err(FError::Quit {
                        message: "Ferium requires an internet connection to work".into(),
                    })
                }
            }
        }
    }

    // Reference to Ferium's config file
    let mut config_file = json::get_config_file().await?;
    // Deserialised config from `config_file`
    let mut config = serde_json::from_reader(&config_file)?;
    // Get the command to execute from Clap
    let command = cli::get_subcommand()?;
    // HTTP(S) client
    let client = Client::new();

    // Run function based on command to be executed
    match command {
        SubCommand::Add { mod_id } => {
            add_mod_modrinth(&client, mod_id, &mut config, &mut config_file).await?;
        }
        SubCommand::AddRepo { owner, name } => {
            add_repo_github(&client, owner, name, &mut config, &mut config_file).await?;
        }
        SubCommand::Remove => {
            check_empty_config(&config)?;
            remove(&client, config, &mut config_file).await?;
        }
        SubCommand::List => {
            check_empty_config(&config)?;
            list(&client, config).await?;
        }
        SubCommand::Upgrade => {
            check_empty_config(&config)?;
            upgrade_modrinth(&client, &config).await?;
            upgrade_github(&client, &config).await?;
        }
    };

    Ok(())
}

/// Check if `config`'s mods and repos are empty, and if so return an error
fn check_empty_config(config: &json::Config) -> FResult<()> {
    if config.repos.is_empty() && config.mod_slugs.is_empty() {
        Err(FError::EmptyConfigFile)
    } else {
        Ok(())
    }
}

/// Display a list of mods and repos in the config to select and remove selected ones from `config_file`
async fn remove(client: &Client, config: json::Config, config_file: &mut File) -> FResult<()> {
    let mut mod_items: Vec<String> = Vec::new();
    let mut repo_items: Vec<String> = Vec::new();
    let mut mods_repos_removed = String::new();
    let mut config = config;

    wrappers::print("Gathering mod and repository information... ");
    // Store the names of the mods
    for i in 0..config.mod_slugs.len() {
        let mod_ = labrinth::calls::get_mod(client, &config.mod_slugs[i]).await?;
        mod_items.push(mod_.title);
    }

    // Store the names of the repos
    for i in 0..config.repos.len() {
        let repo =
            octorok::calls::get_repository(client, &config.repos[i].owner, &config.repos[i].name)
                .await?;
        repo_items.push(repo.full_name);
    }
    println!("✓");

    // If list is not empty
    if !mod_items.is_empty() {
        // Show selection menu
        println!("\nSelect mods to remove");
        let mods_to_remove = MultiSelect::new().items(&mod_items).clear(false).interact();

        match mods_to_remove {
            Ok(mods_to_remove) => {
                // Sort vector in descending order to fix moving indices
                let mut mods_to_remove = mods_to_remove;
                mods_to_remove.sort_unstable();
                mods_to_remove.reverse();

                // For each mod to remove
                for mod_to_remove in mods_to_remove {
                    // Remove it from config and store its name in a string
                    mods_repos_removed.push_str(&config.mod_slugs.remove(mod_to_remove));
                    mods_repos_removed.push_str(", ");
                }
            }
            Err(_) => (),
        }
    }

    // If list is not empty
    if !repo_items.is_empty() {
        // Show selection menu
        println!("\nSelect repositories to remove");
        let repos_to_remove = MultiSelect::new()
            .items(&repo_items)
            .clear(false)
            .interact();

        match repos_to_remove {
            Ok(repos_to_remove) => {
                // Sort vector in descending order to fix moving indices
                let mut repos_to_remove = repos_to_remove;
                repos_to_remove.sort_unstable();
                repos_to_remove.reverse();

                // For each repo to remove
                for repo_to_remove in repos_to_remove {
                    // Remove it from config and store its name in a string
                    mods_repos_removed
                        .push_str(&format!("{}, ", config.repos.remove(repo_to_remove)));
                }
            }
            Err(_) => (),
        }
    }

    // Write updated info to config file
    json::write_to_config(config_file, &config)?;

    // Display mods/repos removed
    if !mods_repos_removed.is_empty() {
        // Remove trailing ", "
        mods_repos_removed.truncate(mods_repos_removed.len() - 2);
        println!("Removed {} from config", mods_repos_removed);
    }

    Ok(())
}

/// Check if https://github.com/{owner}/{repo_name} exists and releases mods, and if so add repo to `config_file`
async fn add_repo_github(
    client: &Client,
    owner: String,
    repo_name: String,
    config: &mut json::Config,
    config_file: &mut File,
) -> FResult<()> {
    // Check if repo has already been added
    if config.repos.contains(&json::Repo {
        owner: owner.clone(),
        name: repo_name.clone(),
    }) {
        return Err(FError::Quit {
            message: "Repo already added to config!".into(),
        });
    }

    wrappers::print(format!("Adding repo {}/{}... ", owner, repo_name));

    // Get repository metadata
    let repo = get_repository(client, &owner, &repo_name).await?;

    // Get the latest release from repo
    let latest_release = &get_releases(client, repo).await?[0];

    let mut contains_jar_asset = false;

    // Check if the latest release contains JAR files (a mod file)
    for asset in &latest_release.assets {
        if asset.name.contains(".jar") {
            // If JAR release is found, set flag to true
            contains_jar_asset = true;
        }
    }

    if contains_jar_asset {
        // Append repo to config and write
        config.repos.push(json::Repo {
            owner,
            name: repo_name,
        });
        json::write_to_config(config_file, config)?;
        println!("✓")
    } else {
        return Err(FError::Quit {
            message: "Repository does not release mods!".into(),
        });
    }

    Ok(())
}

/// Check if mod with `mod_id` exists and releases mods for configured mod loader, and if so add that mod to `config_file`
async fn add_mod_modrinth(
    client: &Client,
    mod_id: String,
    config: &mut json::Config,
    config_file: &mut File,
) -> FResult<()> {
    // Check if mod has already been added
    if config.mod_slugs.contains(&mod_id) {
        return Err(FError::Quit {
            message: "Mod already added to config!".into(),
        });
    }

    wrappers::print(format!("Adding mod {}... ", mod_id));

    // Check if mod exists
    match get_mod(client, &mod_id).await {
        Ok(mod_) => {
            // And if so, append mod to config and write
            config.mod_slugs.push(mod_id);
            json::write_to_config(config_file, config)?;
            println!("✓ ({})", mod_.title);
        }
        Err(_) => {
            // Else return an error
            return Err(FError::Quit {
                message: format!("Mod with ID {} does not exist!", mod_id),
            });
        }
    };

    Ok(())
}

/// List all the mods in `config` with some of their metadata
async fn list(client: &Client, config: json::Config) -> FResult<()> {
    for mod_slug in config.mod_slugs {
        // Get mod metadata
        let mod_ = get_mod(client, &mod_slug).await?;

        // Print mod data formatted
        println!(
            " -  {}
          \r        {}
          \r        Downloads:   {}
          \r        Client side: {}
          \r        Server side: {}
          \r        License:     {}\n",
            mod_.title,
            mod_.description,
            mod_.downloads,
            mod_.client_side,
            mod_.server_side,
            mod_.license.name,
        );
    }

    for repo_name in config.repos {
        // Get repository metadata
        let repo = get_repository(client, &repo_name.owner, &repo_name.name).await?;

        // Print repository data formatted
        println!(
            " -  {}
          \r        {}
          \r        Stars:      {}
          \r        Developer:  {}
          \r        License:    {}\n",
            repo.name, repo.description, repo.stargazers_count, repo.owner.login, repo.license.name,
        )
    }

    Ok(())
}

/// Download and install all the mods in `config.repos`
async fn upgrade_github(client: &Client, config: &json::Config) -> FResult<()> {
    for repo_name in &config.repos {
        println!("Downloading {}", repo_name.name);
        wrappers::print("  [1] Getting release information... ");
        // Get mod's repository
        let repository = get_repository(client, &repo_name.owner, &repo_name.name).await?;
        // Get releases
        let releases = get_releases(client, repository).await?;

        let mut latest_release: Option<&octorok::structs::Release> = None;

        // Try to get the latest compatible release
        for release in &releases {
            if release.name.contains(&config.version) {
                latest_release = Some(release);
                break;
            }
        }

        let latest_release = match latest_release {
            // If a compatible release was found, install it
            Some(release) => {
                println!("✓");
                release
            }
            // If not, default to the latest one
            None => {
                println!(
                    "✓ Warning! Did not find release with version in name. Defaulting to latest"
                );
                &releases[0]
            }
        };

        wrappers::print(format!("  [2] Downloading {}... ", latest_release.name));

        let mut mod_file_path = config.output_dir.join(&repo_name.name);
        mod_file_path.set_extension("jar");

        // Get file contents
        let contents = download_release(client, latest_release).await?;

        // Open the mod JAR file
        let mut mod_file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(mod_file_path)?;

        // Write download to mod JAR file
        mod_file.write_all(&contents)?;
        println!("✓\n");
    }

    Ok(())
}

/// Download and install all mods in `config.mod_slugs`
async fn upgrade_modrinth(client: &Client, config: &json::Config) -> FResult<()> {
    for mod_slug in &config.mod_slugs {
        // Get mod metadata
        let mod_ = get_mod(client, &mod_slug).await?;
        println!("Downloading {}", mod_.title);

        wrappers::print("  [1] Getting version information... ");
        // Get versions of the mod
        let versions = get_versions(client, &mod_.id).await?;

        let mut latest_version: Option<labrinth::structs::Version> = None;

        // Check if a version compatible with the game version specified in the config is available
        for version in versions {
            if version.game_versions.contains(&config.version) {
                latest_version = Some(version);
                break;
            }
        }

        let latest_version = match latest_version {
            Some(version) => version,
            // If version compatible with game version does not exist, quit with error
            None => {
                return Err(FError::Quit {
                    message: format!(
                        "No version of {} is compatible for Minecraft {}",
                        mod_.title, config.version,
                    ),
                });
            }
        };

        println!("✓");

        wrappers::print(format!("  [2] Downloading {}... ", latest_version.name));

        let mut mod_file_path = config.output_dir.join(mod_.title);
        mod_file_path.set_extension("jar");

        // Get file contents
        let contents = download_version(client, latest_version).await?;

        // Open mod JAR file
        let mut mod_file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(mod_file_path)?;

        // Write contents to JAR file
        mod_file.write_all(&contents)?;
        println!("✓\n");
    }

    Ok(())
}
