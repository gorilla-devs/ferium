//! This file contains convenience wrappers for configurations and general JSON stuff

use super::wrappers::get_mods_dir;
use crate::ferium_error::{FError, FResult};
use dialoguer::{Confirm, Select};
use native_dialog::FileDialog;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use shellexpand::tilde;
use std::cmp::PartialEq;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    /// The directory to download mod JARs to
    pub output_dir: PathBuf,
    /// Check if versions/releases are compatible with this Minecraft version
    pub version: String,
    /// Check if versions/releases are compatible with this mod loader
    pub loader: String,
    /// A list of mod slugs of mods to download
    pub mod_slugs: Vec<String>,
    /// A list of repositories of mods to download
    pub repos: Vec<Repo>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
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
pub async fn get_config_file() -> FResult<File> {
    // Get home directory
    let home: PathBuf = tilde("~").as_ref().into();

    // Config file's path
    let config_file_path = home.join(".config").join("ferium").join("config.json");

    // If config file exists
    if config_file_path.exists() {
        // Open and return it
        Ok(OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(false)
            .open(config_file_path)?)

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

        println!("Welcome to Ferium, your easy to use Minecraft mod manager.\nThis seems to be your first time using Ferium, so we are going go through some settings");

        // Let user pick mods directory
        let mut selected_mods_dir = get_mods_dir()?;
        println!(
            "\nThe default mods directory is {}",
            selected_mods_dir.to_str().ok_or(FError::OptionError)?
        );
        if Confirm::new()
            .with_prompt("Would you like to specify a custom mods directory?")
            .interact()?
        {
            match FileDialog::new().show_open_single_dir()? {
                Some(dir) => selected_mods_dir = dir,
                None => (),
            };
        }

        create_dir_all(&selected_mods_dir)?;

        // Let user pick Minecraft version
        let mut latest_versions: Vec<String> = super::wrappers::get_latest_mc_versions(20).await?;
        println!("\nWhich version of Minecraft do you play?");
        let selected_version = Select::new()
            .items(&latest_versions)
            .clear(false)
            .default(0)
            .interact()?;
        let selected_version = latest_versions.swap_remove(selected_version);

        // Let user pick mod loader
        let mod_loaders = ["fabric", "forge"];
        println!("\nWhich mod loader do you use?");
        let selected_loader = mod_loaders[Select::new()
            .items(&mod_loaders)
            .clear(false)
            .default(0)
            .interact()?];

        // Write to config file with configured values
        write_to_config(
            &mut file,
            &Config {
                output_dir: selected_mods_dir,
                mod_slugs: Vec::new(),
                repos: Vec::new(),
                version: selected_version,
                loader: selected_loader.into(),
            },
        )?;

        Err(FError::Quit {
            message: "First time setup complete".into(),
        })
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
