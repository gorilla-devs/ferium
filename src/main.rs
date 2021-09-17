mod labrinth;
mod octorok;
mod util;

use labrinth::calls::*;
use octorok::calls::*;
use reqwest::Client;
use std::fs::{File, OpenOptions};
use std::io::Write;
use util::{cli::SubCommand, *};

#[tokio::main]
async fn main() {
    // Reference to Ferium's config file
    let mut config_file = json::get_config_file();
    // Config deserialised from `config_file`
    let mut config = serde_json::from_reader(&config_file).unwrap();
    // Get command to execute from Clap
    let command = cli::get_subcommand();
    // HTTP client
    let client: Client = Client::new();

    match command {
        SubCommand::Add { mod_id } => {
            add_mod_modrinth(&client, mod_id, &mut config, &mut config_file).await
        }
        SubCommand::AddRepo { owner, name } => {
            add_repo_github(&client, owner, name, &mut config, &mut config_file).await;
        }
        SubCommand::List => list(&client, config).await,
        SubCommand::Upgrade => {
            upgrade_modrinth(&client, &config).await;
            upgrade_github(&client, &config).await;
        }
    };
}

/// Check if https://github.com/{owner}/{repo_name} exists and releases mods, and if so add repo to `config_file`
async fn add_repo_github(
    client: &Client,
    owner: String,
    repo_name: String,
    config: &mut json::Config,
    config_file: &mut File,
) {
    // Check if repo has already been added
    if config.repos.contains(&json::Repo {
        owner: owner.clone(),
        name: repo_name.clone(),
    }) {
        panic!("Repo already added to config!");
    }

    wrappers::print(format!("Adding repo {}/{}...", owner, repo_name));

    // Get repository metadata
    let repo = get_repository(client, &owner, &repo_name).await;

    // Get the latest release from repo
    let latest_release = &get_releases(client, repo).await[0];

    let mut contains_jar_asset = false;

    // Check if the latest release contains JAR files (a mod file)
    for asset in &latest_release.assets {
        if asset.content_type.contains("java") {
            contains_jar_asset = true;
        }
    }

    if contains_jar_asset {
        // Append repo to config and write
        config.repos.push(json::Repo {
            owner: owner,
            name: repo_name,
        });
        json::write_to_config(config_file, config);
        println!("✓")
    } else {
        panic!("Repository does not release mods!");
    }
}

/// Check if mod with `mod_id` exists, and if so, add that mod to `config_file`
async fn add_mod_modrinth(
    client: &Client,
    mod_id: String,
    config: &mut json::Config,
    config_file: &mut File,
) {
    // Check if mod has already been added
    if config.mod_slugs.contains(&mod_id) {
        panic!("Mod already added to config!");
    }

    wrappers::print(format!("Adding mod {}... ", mod_id));

    // Check if mod exists
    if let Some(mod_) = does_exist(client, &mod_id).await {
        // And if so, append mod to config and write
        config.mod_slugs.push(mod_id);
        json::write_to_config(config_file, config);
        println!("✓ ({})", mod_.title);
    } else {
        panic!("Mod with ID {} does not exist!", mod_id);
    }
}

/// List all the mods in `config` and some of their metadata
async fn list(client: &Client, config: json::Config) {
    // Check if mods and repos are empty, and if so tell user to add mods or repos
    if config.mod_slugs.is_empty() && config.mod_slugs.is_empty() {
        panic!(
            "Your config file contains no mods or repos! Run `ferium help` to see how to add mods or repos."
        );
    }

    for mod_slug in config.mod_slugs {
        // Get mod metadata
        let mod_ = get_mod(client, &mod_slug).await;

        // Print mod data nicely formatted
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
        let repo = get_repository(client, &repo_name.owner, &repo_name.name).await;

        // Print repository data nicely formatted
        println!(
            " - {}
          \r        {}
          \r        Stars:      {}
          \r        Developer:  {}
          \r        License:    {}\n",
            repo.name, repo.description, repo.stargazers_count, repo.owner.login, repo.license.name,
        )
    }
}

/// Download and install all the mods in `config.repos`
async fn upgrade_github(client: &Client, config: &json::Config) {
    // Check if empty and tell user to add mods
    if config.repos.is_empty() {
        panic!("Your config file contains no repos! Run `ferium help` to see how to add repos.");
    }

    for repo_name in &config.repos {
        println!("Downloading {}", repo_name.name);
        wrappers::print("  [1] Getting release information... ");
        // Get mod's repository
        let repository = get_repository(client, &repo_name.owner, &repo_name.name).await;
        // Get releases. Index 0 is the latest release because of chronological ordering
        let latest_release = &get_releases(client, repository).await[0];
        println!("✓");

        // Open the local mod JAR file
        let mut mod_jar = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!("{}{}.jar", config.output_dir, repo_name.name))
            .unwrap();

        // Download file
        wrappers::print(format!("  [2] Downloading {}... ", latest_release.name));
        let contents = download_release(client, latest_release).await;
        println!("✓");

        // Write download to JAR file
        mod_jar.write_all(&contents).unwrap();
        println!("");
    }
}

/// Download and install all mods in `config.mod_slugs`
async fn upgrade_modrinth(client: &Client, config: &json::Config) {
    // Check if empty and tell user to add mods
    if config.mod_slugs.is_empty() {
        panic!("Your config file contains no mods! Run `ferium help` to see how to add mods.");
    }

    for mod_slug in &config.mod_slugs {
        // Get mod metadata
        let mod_ = get_mod(client, &mod_slug).await;
        println!("Downloading {}", mod_.title);

        // Get versions of the mod
        wrappers::print("  [1] Getting version information... ");
        let versions = get_versions(client, &mod_.id).await;
        println!("✓");

        // Versions are arranged chronologically so first one is the latest
        let latest_version = &versions[0];

        // Open mod JAR file
        let mut mod_jar = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!("{}{}.jar", config.output_dir, mod_.title))
            .unwrap();

        // Download file
        wrappers::print(format!("  [2] Downloading {}... ", latest_version.name));
        let contents = download_version(client, latest_version).await;
        println!("✓");

        // Write download to JAR file
        mod_jar.write_all(&contents).unwrap();
        println!("");
    }
}
