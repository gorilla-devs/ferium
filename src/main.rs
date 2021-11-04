mod octorok;
mod util;

use ansi_term::Colour::{Green, White};
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use ferinth::{structures::*, Ferinth};
use octorok::calls::*;
use reqwest::Client;
use std::{
    fs::{create_dir_all, remove_file, OpenOptions},
    io::Write,
    path::PathBuf,
};
use util::{
    cli::SubCommand,
    ferium_error::{FError, FResult},
    *,
};

#[tokio::main]
async fn main() -> FResult<()> {
    // Get the command to execute from Clap
    // This also displays help, version
    let command = cli::get_subcommand()?;

    // Check for an internet connection
    match online::check(Some(1)).await {
        Ok(_) => (),
        Err(_) => {
            eprint!("Checking internet connection... ");
            match online::check(Some(4)).await {
                Ok(_) => println!("✓"),
                Err(_) => {
                    return Err(FError::Quit {
                        message: "× Ferium requires an internet connection to work".into(),
                    })
                }
            }
        }
    }

    // HTTP(S) client
    let client = Client::new();
    let modrinth = Ferinth::new("ferium");
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
            add_mod_modrinth(&modrinth, mod_id, &mut config).await?;
        }
        SubCommand::AddRepo { owner, name } => {
            add_repo_github(&client, owner, name, &mut config).await?;
        }
        SubCommand::Config => {
            configure(&mut config).await?;
        }
        SubCommand::List => {
            check_empty_config(&config)?;
            list(&modrinth, &client, &config).await?;
        }
        SubCommand::Remove => {
            check_empty_config(&config)?;
            remove(&modrinth, &client, &mut config).await?;
        }
        SubCommand::Upgrade => {
            check_empty_config(&config)?;
            create_dir_all(&config.output_dir)?;
            upgrade_modrinth(&modrinth, &config).await?;
            upgrade_github(&client, &config).await?
        }
    };

    // Update config file with new values
    json::write_to_config(&mut config_file, &config)?;

    Ok(())
}

/// Fetch a mod file's path based on a `name` and the `config`
fn get_mod_file_path(config: &json::Config, name: &str) -> PathBuf {
    let mut mod_file_path = config.output_dir.join(format!("{}", name));
    mod_file_path.set_extension("jar");
    mod_file_path
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
        // Quit the configuration
        "Quit",
    ];

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Which setting would you like to change")
            .items(&items)
            .interact_opt()?;

        match selection {
            Some(index) => {
                match index {
                    0 => {
                        eprint!(
                            "{} {}",
                            Green.paint("✔"),
                            White.bold().paint("Pick a mod output directory   "),
                        );
                        // Let user pick output directory
                        match wrappers::pick_folder(&config.output_dir).await {
                            Some(dir) => config.output_dir = dir,
                            None => (),
                        }
                        println!(
                            "{}\n",
                            Green.paint(config.output_dir.to_str().ok_or(FError::OptionError)?)
                        );
                    }
                    1 => {
                        // Let user pick mc version from latest 10 versions
                        let mut versions = wrappers::get_latest_mc_versions(10).await?;
                        let index = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Select a Minecraft version")
                            .items(&versions)
                            .default(0)
                            .interact_opt()?;
                        match index {
                            Some(i) => config.version = versions.swap_remove(i),
                            None => (),
                        }
                        println!();
                    }
                    2 => {
                        // Let user pick mod loader
                        let mod_loaders = ["Fabric", "Forge"];
                        let index = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Pick a mod loader")
                            .items(&mod_loaders)
                            .interact_opt()?;
                        match index {
                            Some(i) => config.loader = mod_loaders[i].to_lowercase(),
                            None => (),
                        }
                        println!();
                    }
                    3 => break,
                    _ => return Err(FError::OptionError),
                }
            }
            None => break,
        };
    }

    Ok(())
}

/// Display a list of mods and repos in the config to select and remove selected ones from `config_file`
async fn remove(modrinth: &Ferinth, client: &Client, config: &mut json::Config) -> FResult<()> {
    let mut items: Vec<String> = Vec::new();
    let mut items_removed = String::new();

    eprint!("Gathering mod and repository information... ");
    // Store the names of the mods
    for i in 0..config.mod_slugs.len() {
        let mod_ = modrinth.get_mod(&config.mod_slugs[i]).await?;
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
        let items_to_remove = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select mods and/or repositories to remove")
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
                        let mod_file_path = get_mod_file_path(config, name);
                        let _ = remove_file(mod_file_path);

                        // Store its name in a string
                        items_removed.push_str(&format!("{}, ", name));
                    } else {
                        // Remove item from config
                        config.mod_slugs.remove(item_to_remove);
                        // Get the item's name
                        let name = &items[item_to_remove];

                        // Remove the mod from downloaded mods
                        let mod_file_path = get_mod_file_path(config, name);
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
        println!("Removed {}", items_removed);
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
    eprint!("Adding repo {}/{}... ", owner, repo_name);

    // Check if repo has already been added
    if config.repos.contains(&json::Repo {
        owner: owner.clone(),
        name: repo_name.clone(),
    }) {
        return Err(FError::Quit {
            message: "× Repsitory already added to config!".into(),
        });
    }

    // Get repository metadata
    let repo = get_repository(client, &owner, &repo_name).await?;

    // Get repo's releases
    let releases = &get_releases(client, &repo).await?;

    let mut contains_jar_asset = false;

    // Check if the releases contain JAR files (a mod file)
    'outer: for release in releases {
        for asset in &release.assets {
            if asset.name.contains("jar") {
                // If JAR release is found, set flag to true and break
                contains_jar_asset = true;
                break 'outer;
            }
        }
    }

    if contains_jar_asset {
        // Append repo to config and write
        config.repos.push(json::Repo {
            owner: repo.owner.login,
            name: repo.name,
        });
        println!("✓")
    } else {
        return Err(FError::Quit {
            message: "× Repository does not release mods!".into(),
        });
    }

    Ok(())
}

/// Check if mod with `mod_id` exists and releases mods for configured mod loader, and if so add that mod to `config`
async fn add_mod_modrinth(
    modrinth: &Ferinth,
    mod_id: String,
    config: &mut json::Config,
) -> FResult<()> {
    eprint!("Adding mod ID {}... ", mod_id);

    // Check if mod has already been added
    if config.mod_slugs.contains(&mod_id) {
        return Err(FError::Quit {
            message: "× Mod already added to config!".into(),
        });
    }

    // Check if mod exists
    match modrinth.get_mod(&mod_id).await {
        Ok(mod_) => {
            // And if so, append mod to config and write
            config.mod_slugs.push(mod_id);
            println!("✓ ({})", mod_.title);
        }
        Err(_) => {
            // Else return an error
            return Err(FError::Quit {
                message: format!("× Mod with ID `{}` does not exist!", mod_id),
            });
        }
    };

    Ok(())
}

/// List all the mods in `config` with some of their metadata
async fn list(modrinth: &Ferinth, client: &Client, config: &json::Config) -> FResult<()> {
    for mod_slug in &config.mod_slugs {
        // Get mod metadata
        let mod_ = modrinth.get_mod(&mod_slug).await?;
        let team_members = modrinth.list_team_members(&mod_.team).await?;

        // Get the usernames of all the developers
        let mut developers = String::new();
        for member in team_members {
            let user = modrinth.get_user(&member.user_id).await?;
            developers.push_str(&user.username);
            developers.push_str(", ");
        }
        // Trim trailing ', '
        developers.truncate(developers.len() - 2);

        // Print mod data formatted
        println!(
            "{}
           \r   {}\n
           \r   Link:           https://modrinth.com/mod/{}
           \r   Source:         Modrinth Mod
           \r   Downloads:      {}
           \r   Developers:     {}
           \r   Client side:    {}
           \r   Server side:    {}
           \r   License:        {}\n",
            mod_.title,
            mod_.description,
            mod_.slug,
            mod_.downloads,
            developers,
            mod_.client_side,
            mod_.server_side,
            mod_.license.name,
        );
    }

    for repo_name in &config.repos {
        // Get repository metadata
        let repo = get_repository(client, &repo_name.owner, &repo_name.name).await?;
        let releases = get_releases(client, &repo).await?;
        let mut downloads = 0;

        // Calculate number of downloads
        for release in releases {
            for asset in release.assets {
                downloads += asset.download_count;
            }
        }

        // Print repository data formatted
        println!(
            "{}
           \r   {}\n
           \r   Link:       {}
           \r   Source:     GitHub Repository
           \r   Downloads:  {}
           \r   Developer:  {}
           \r   License:    {}\n",
            repo.name,
            repo.description,
            repo.html_url,
            downloads,
            repo.owner.login,
            repo.license.name,
        )
    }

    Ok(())
}

/// Download and install all the mods in `config.repos`
async fn upgrade_github(client: &Client, config: &json::Config) -> FResult<()> {
    for repo_name in &config.repos {
        println!("Downloading {}", repo_name.name);
        eprint!("  [1] Getting release information... ");

        let repository = get_repository(client, &repo_name.owner, &repo_name.name).await?;
        let releases = get_releases(client, &repository).await?;
        let version_to_check = wrappers::remove_minor_version(&config.version)?;

        // A vector of assets that are compatible
        let mut asset_candidates: Vec<&octorok::structs::Asset> = Vec::new();
        // Whether the mod specifies the mod loader in its Assets' names
        let mut specifies_loader = false;

        // Try to get the latest compatible assets
        for release in &releases {
            // If a release with compatible assets has been found, stop searching older releases
            if !asset_candidates.is_empty() {
                break;
            }

            for asset in &release.assets {
                // If the asset specifies the mod loader, set the `specifies_loader` flag to true
                if asset.name.to_lowercase().contains("fabric")
                    || asset.name.to_lowercase().contains("forge")
                {
                    specifies_loader = true;
                }

                if
                // Check that the asset supports the user's specified version
                (asset.name.contains(&version_to_check) || release.body.contains(&version_to_check))
                    // Check that the asset is a JAR file
                    && asset.name.contains("jar")
                    // If the asset specifies a mod loader, check for it, if not don't check and return true
                    && !(specifies_loader && !asset.name.to_lowercase().contains(&config.loader))
                {
                    // Specify this asset as a compatible asset
                    asset_candidates.push(asset);
                }
            }
        }

        // If 1 compatible asset was found, use it
        let asset_to_download = if asset_candidates.len() == 1 {
            println!("✓");
            asset_candidates[0]
        // If none were found, throw an error
        } else if asset_candidates.len() == 0 {
            return Err(FError::Quit {
                message: "× Could not find a compatible asset to download".into(),
            });
        // If more than 1 was found, let the user select which one to use
        } else {
            println!("✓");
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select the asset to downloaded")
                .items(&asset_candidates)
                .interact()?;
            asset_candidates[selection]
        };

        eprint!("  [2] Downloading {}... ", asset_to_download.name);

        // Compute output mod file's path
        let mod_file_path = get_mod_file_path(config, &repository.name);

        // Get file contents
        let contents = download_asset(client, asset_to_download).await?;

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
async fn upgrade_modrinth(modrinth: &Ferinth, config: &json::Config) -> FResult<()> {
    for mod_slug in &config.mod_slugs {
        // Get mod metadata
        let mod_ = modrinth.get_mod(&mod_slug).await?;
        println!("Downloading {}", mod_.title);

        eprint!("  [1] Getting version information... ");
        // Get versions of the mod
        let versions = modrinth.list_versions(&mod_.id).await?;

        let mut latest_version: Option<version_structs::Version> = None;

        // Check if a version compatible with the game version and mod loader specified in the config is available
        for version in versions {
            let mut compatible_version = false;

            for v in &version.game_versions {
                if v.contains(&wrappers::remove_minor_version(&config.version)?) {
                    compatible_version = true;
                }
            }

            if compatible_version && version.loaders.contains(&config.loader) {
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
                        "× No version of {} is compatible for {} {}",
                        mod_.title, config.loader, config.version,
                    ),
                });
            }
        };

        println!("✓");

        eprint!("  [2] Downloading {}... ", latest_version.name);

        // Compute output mod file's path
        let mod_file_path = get_mod_file_path(config, &mod_.title);

        // Get file contents
        let contents = modrinth
            .download_version_file(&latest_version.files[0])
            .await?;

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
