//! This file contains convenience wrappers for configurations and general JSON stuff

use super::wrappers::get_mods_dir;
use crate::ferium_error::{FError, FResult};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use shellexpand::tilde;
use std::cmp::PartialEq;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct Config {
    /// The directory to download mod JARs to
    pub output_dir: PathBuf,
    /// Check if versions/releases are compatible with this Minecraft version
    pub version: String,
    /// Check if versions/releases are compatible with this mod loader
    pub loader: String,
    /// A list of mod slugs of Modrinth mods to download
    pub mod_slugs: Vec<String>,
    /// A list GitHub repositories to download
    pub repos: Vec<Repo>,
}

impl Config {
    /// Run a first time setup where the user picks the settings
    async fn new() -> FResult<Self> {
        println!("Welcome to Ferium, your easy to use Minecraft mod manager.");
        println!("This seems to be your first time using Ferium, so we are going go through some settings.\n");

        // Let user pick mods directory
        let mut selected_mods_dir = get_mods_dir()?;
        println!(
            "The default mods directory is {}",
            selected_mods_dir.to_str().ok_or(FError::OptionError)?
        );
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Would you like to specify a custom mods directory?")
            .interact()?
        {
            if let Some(dir) = super::wrappers::pick_folder(&selected_mods_dir).await {
                selected_mods_dir = dir;
            };
        }

        // Let user pick Minecraft version
        let mut latest_versions: Vec<String> = super::wrappers::get_latest_mc_versions(10).await?;
        println!();
        let selected_version = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Which version of Minecraft do you play?")
            .items(&latest_versions)
            .default(0)
            .interact()?;
        let selected_version = latest_versions.swap_remove(selected_version);

        // Let user pick mod loader
        let mod_loaders = ["Fabric", "Forge"];
        let selected_loader = mod_loaders[Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Which mod loader do you use?")
            .items(&mod_loaders)
            .default(0)
            .interact()?];

        // Return config with the configured values
        println!("First time setup complete!");
        Ok(Config {
            output_dir: selected_mods_dir,
            mod_slugs: Vec::new(),
            repos: Vec::new(),
            version: selected_version,
            loader: selected_loader.to_lowercase(),
        })
    }
}

#[derive(Deserialize, Serialize, PartialEq)]
pub struct Repo {
    /// Username of the owner of the repository
    pub owner: String,
    /// Name of the repository
    pub name: String,
}

impl std::fmt::Display for Repo {
    fn fmt(&self, fmter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmter, "{}/{}", self.owner, self.name)
    }
}

/// Returns the config file. If not found, first time setup will run and write picked values to the config file
pub async fn get_config_file() -> FResult<Option<File>> {
    // Home directory
    let home: PathBuf = tilde("~").as_ref().into();
    // Config file's path
    let config_file_path = home.join(".config").join("ferium").join("config.json");

    // If config file exists
    if config_file_path.exists() {
        // Open and return it
        Ok(Some(
            OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(false)
                .create(false)
                .open(config_file_path)?,
        ))

    // If config file does not exist
    } else {
        // Make sure config directory exists
        create_dir_all(config_file_path.parent().ok_or(FError::OptionError)?)?;

        // Create and open config file
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(config_file_path)?;

        // Write to config file with values from first time setup
        write_to_config(&mut file, &Config::new().await?)?;
        Ok(None)
    }
}

/// Serialise and write `config` to `config_file`
pub fn write_to_config(config_file: &mut File, config: &Config) -> FResult<()> {
    // Serialise config
    let contents = to_string_pretty(&config)?;

    // Erase the file
    config_file.set_len(0)?;
    config_file.seek(SeekFrom::Start(0))?;

    // Write config to file
    config_file.write_all(contents.as_bytes())?;

    Ok(())
}
