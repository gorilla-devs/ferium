mod labrinth;
mod octorok;
mod util;

use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use labrinth::calls::*;
use octorok::calls::*;
use reqwest::Client;
use std::{
    fs::{create_dir_all, remove_file, OpenOptions},
    io::Write,
};
use util::{
    cli::SubCommand,
    ferium_error::{FError, FResult},
    *,
};

#[tokio::main]
async fn main() -> FResult<()> {
    // Check for an internet connection
    match online::check(Some(1)).await {
        Ok(_) => (),
        Err(_) => {
            eprint!("Checking internet connection... ");
            match online::check(Some(4)).await {
                Ok(_) => println!("✓"),
                Err(_) => {
                    return Err(FError::Quit {
                        message: "Ferium requires an internet connection to work".into(),
                    })
                }
            }
        }
    }

    // Get the command to execute from Clap
    let command = cli::get_subcommand()?;
    // HTTP(S) client
    let client = Client::new();
    // Reference to Ferium's config file
    let mut config_file = match json::get_config_file().await? {
        Some(file) => file,
        // Exit program if first time setup ran
        None => return Ok(()),
    };
    // Deserialised config from `config_file`
    let mut config = serde_json::from_reader(&config_file)?;

    // Run function(s) based on command to be executed
    match command {
        SubCommand::Add { mod_id } => {
            add_mod_modrinth(&client, mod_id, &mut config).await?;
        }
        SubCommand::AddRepo { owner, name } => {
            add_repo_github(&client, owner, name, &mut config).await?;
        }
        SubCommand::Config => {
            configure(&mut config).await?;
        }
        SubCommand::List => {
            check_empty_config(&config)?;
            list(&client, &config).await?;
        }
        SubCommand::Remove => {
            check_empty_config(&config)?;
            remove(&client, &mut config).await?;
        }
        SubCommand::Upgrade => {
            check_empty_config(&config)?;
            create_dir_all(&config.output_dir)?;
            upgrade_modrinth(&client, &config).await?;
            upgrade_github(&client, &config).await?;
        }
    };

    // Update config file with new values
    json::write_to_config(&mut config_file, &config)?;

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

async fn configure(config: &mut json::Config) -> FResult<()> {
    let items = vec![
        // Show dialog to change directory
        "Mods output directory",
        // Show picker to change Minecraft version
        "Minecraft version",
        // Show picker to change mod loader
        "Mod loader",
    ];

    println!("Which setting would you like to change?");
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .interact_opt()?;

    match selection {
        Some(index) => {
            println!();
            match index {
                0 => {
                    // Let user pick output directory
                    match wrappers::pick_folder().await {
                        Some(dir) => config.output_dir = dir,
                        None => (),
                    }
                }
                1 => {
                    // Let user pick mc version from latest 10 versions
                    let mut versions = wrappers::get_latest_mc_versions(10).await?;
                    println!("Select a Minecraft version");
                    let index = Select::with_theme(&ColorfulTheme::default())
                        .items(&versions)
                        .default(0)
                        .interact_opt()?;
                    match index {
                        Some(i) => config.version = versions.swap_remove(i),
                        None => (),
                    }
                }
                2 => {
                    // Let user pick mod loader
                    let mod_loaders = ["Fabric", "Forge"];
                    println!("Pick a mod loader");
                    let index = Select::with_theme(&ColorfulTheme::default())
                        .items(&mod_loaders)
                        .interact_opt()?;
                    match index {
                        Some(i) => config.loader = mod_loaders[i].to_lowercase(),
                        None => (),
                    }
                }
                _ => return Err(FError::OptionError),
            }
        }
        None => return Ok(()),
    };

    Ok(())
}

/// Display a list of mods and repos in the config to select and remove selected ones from `config_file`
async fn remove(client: &Client, config: &mut json::Config) -> FResult<()> {
    let mut items: Vec<String> = Vec::new();
    let mut items_removed = String::new();

    eprint!("Gathering mod and repository information... ");
    // Store the names of the mods
    for i in 0..config.mod_slugs.len() {
        let mod_ = labrinth::calls::get_mod(client, &config.mod_slugs[i]).await?;
        items.push(mod_.title);
    }

    // Store the names of the repos
    for i in 0..config.repos.len() {
        let repo =
            octorok::calls::get_repository(client, &config.repos[i].owner, &config.repos[i].name)
                .await?;
        items.push(repo.name);
    }
    println!("✓");

    // If list is not empty
    if !items.is_empty() {
        // Show selection menu
        println!("\nSelect mods and/or repositories to remove");
        let items_to_remove = MultiSelect::with_theme(&ColorfulTheme::default())
            .items(&items)
            .clear(false)
            .interact_opt()?;

        match items_to_remove {
            Some(items_to_remove) => {
                // Sort vector in descending order to fix moving indices
                let mut items_to_remove = items_to_remove;
                items_to_remove.sort_unstable();
                items_to_remove.reverse();

                // For each mod to remove
                for item_to_remove in items_to_remove {
                    // If index is larger than mod_slugs length, then the index is for repos
                    if item_to_remove >= config.mod_slugs.len() {
                        // Offset the array by the proper amount
                        let index = item_to_remove - config.mod_slugs.len();

                        // Remove item from config
                        config.repos.remove(index);
                        // Get the item's name
                        let name = &items[item_to_remove];

                        // Remove the mod from downloaded mods
                        let mut mod_file_path = config.output_dir.join(name);
                        mod_file_path.set_extension("jar");
                        let _ = remove_file(mod_file_path);

                        // Store its name in a string
                        items_removed.push_str(&format!("{}, ", name));
                    } else {
                        // Remove item from config
                        config.mod_slugs.remove(item_to_remove);
                        // Get the item's name
                        let name = &items[item_to_remove];

                        // Remove the mod from downloaded mods
                        let mut mod_file_path = config.output_dir.join(name);
                        mod_file_path.set_extension("jar");
                        let _ = remove_file(mod_file_path);

                        // Store its name in a string
                        items_removed.push_str(&format!("{}, ", name));
                    }
                }
            }

            // Exit if none are selected
            None => (),
        }
    }

    // Display mods/repos removed
    if !items_removed.is_empty() {
        // Remove trailing ", "
        items_removed.truncate(items_removed.len() - 2);
        println!("Removed {} from config", items_removed);
    }

    Ok(())
}

/// Check if https://github.com/{owner}/{repo_name} exists and releases mods, and if so add repo to `config_file`
async fn add_repo_github(
    client: &Client,
    owner: String,
    repo_name: String,
    config: &mut json::Config,
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

    eprint!("Adding repo {}/{}... ", owner, repo_name);

    // Get repository metadata
    let repo = get_repository(client, &owner, &repo_name).await?;

    // Get repo's releases
    let releases = &get_releases(client, &repo).await?;

    let mut contains_jar_asset = false;

    // Check if the releases contain JAR files (a mod file)
    'outer: for release in releases {
        for asset in &release.assets {
            if asset.name.contains(".jar") {
                // If JAR release is found, set flag to true and break
                contains_jar_asset = true;
                break 'outer;
            }
        }
    }

    if contains_jar_asset {
        // Append repo to config and write
        config.repos.push(json::Repo {
            owner,
            name: repo_name,
        });
        println!("✓")
    } else {
        return Err(FError::Quit {
            message: "Repository does not release mods!".into(),
        });
    }

    Ok(())
}

/// Check if mod with `mod_id` exists and releases mods for configured mod loader, and if so add that mod to `config`
async fn add_mod_modrinth(
    client: &Client,
    mod_id: String,
    config: &mut json::Config,
) -> FResult<()> {
    // Check if mod has already been added
    if config.mod_slugs.contains(&mod_id) {
        return Err(FError::Quit {
            message: "Mod already added to config!".into(),
        });
    }

    eprint!("Adding mod {}... ", mod_id);

    // Check if mod exists
    match get_mod(client, &mod_id).await {
        Ok(mod_) => {
            // And if so, append mod to config and write
            config.mod_slugs.push(mod_id);
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
async fn list(client: &Client, config: &json::Config) -> FResult<()> {
    for mod_slug in &config.mod_slugs {
        // Get mod metadata
        let mod_ = get_mod(client, &mod_slug).await?;

        // Print mod data formatted
        println!(
            "- {}
          \r       {}
          \r       Downloads:   {}
          \r       Client side: {}
          \r       Server side: {}
          \r       License:     {}\n",
            mod_.title,
            mod_.description,
            mod_.downloads,
            mod_.client_side,
            mod_.server_side,
            mod_.license.name,
        );
    }

    for repo_name in &config.repos {
        // Get repository metadata
        let repo = get_repository(client, &repo_name.owner, &repo_name.name).await?;

        // Print repository data formatted
        println!(
            "- {}
          \r       {}
          \r       Stars:      {}
          \r       Developer:  {}
          \r       License:    {}\n",
            repo.name, repo.description, repo.stargazers_count, repo.owner.login, repo.license.name,
        )
    }

    Ok(())
}

/// Download and install all the mods in `config.repos`
async fn upgrade_github(client: &Client, config: &json::Config) -> FResult<()> {
    for repo_name in &config.repos {
        println!("Downloading {}", repo_name.name);
        eprint!("  [1] Getting release information... ");
        // Get mod's repository
        let repository = get_repository(client, &repo_name.owner, &repo_name.name).await?;
        // Get releases
        let releases = get_releases(client, &repository).await?;

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

        eprint!("  [2] Downloading {}... ", latest_release.name);

        // Compute mod file's output path
        let mut mod_file_path = config.output_dir.join(&repository.name);
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

        eprint!("  [1] Getting version information... ");
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

        eprint!("  [2] Downloading {}... ", latest_version.name);

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
