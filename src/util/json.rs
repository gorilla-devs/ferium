//! Contains convenience wrappers for the configuration/settings

use super::wrappers::get_mods_dir;
use crate::ferium_error::{FError, FResult};
use clap::ArgEnum;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use shellexpand::tilde;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct Profile {
	/// The profile's name
	pub name: String,
	/// The directory to download mod JARs to
	pub output_dir: PathBuf,
	/// Check if versions/releases are compatible with this Minecraft version
	pub game_version: String,
	/// Check if versions/releases are compatible with this mod loader
	pub mod_loader: ModLoaders,
	/// A list of mod slugs/IDs of Modrinth mods to download
	pub mod_ids: Vec<String>,
	/// A list GitHub repositories to download
	pub repos: Vec<(String, String)>,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
	// The index of the active profile
	pub active_profile: usize,
	// The profiles
	pub profiles: Vec<Profile>,
}

#[derive(ArgEnum, Clone, Deserialize, Serialize, Debug)]
pub enum ModLoaders {
	Fabric,
	Forge,
}

impl std::fmt::Display for ModLoaders {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)?;

		Ok(())
	}
}

impl Profile {
	/// Run a first time setup where the user picks the settings through a UI
	pub async fn create_ui(config: &Config) -> FResult<Self> {
		// Let user pick mods directory
		let mut selected_mods_dir = get_mods_dir()?;
		println!(
			"The default mods directory is {:?}",
			selected_mods_dir
		);
		if Confirm::with_theme(&ColorfulTheme::default())
			.with_prompt("Would you like to specify a custom mods directory?")
			.interact()?
		{
			if let Some(dir) = super::wrappers::pick_folder(&selected_mods_dir).await {
				selected_mods_dir = dir;
			};
		}

		let mut name = String::new();
		let mut prompt = true;
		while prompt {
			name = Input::with_theme(&ColorfulTheme::default())
				.with_prompt("What should this profile be called? ")
				.interact_text()?;

			prompt = false;
			for profile in &config.profiles {
				if profile.name == name {
					println!("A profile with name {} already exists!", name);
					prompt = true;
				}
			}
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
		let selected_loader = if Select::with_theme(&ColorfulTheme::default())
			.with_prompt("Which mod loader do you use?")
			.items(&mod_loaders)
			.interact()? == 0
		{
			ModLoaders::Fabric
		} else {
			ModLoaders::Forge
		};

		// Return config with the configured values
		Ok(Self {
			name,
			output_dir: selected_mods_dir,
			mod_ids: Vec::new(),
			repos: Vec::new(),
			game_version: selected_version,
			mod_loader: selected_loader,
		})
	}
}

/// Get the path to the config file
pub fn get_config_file_path() -> PathBuf {
	// Home directory
	let home: PathBuf = tilde("~").as_ref().into();
	home.join(".config").join("ferium").join("config.json")
}

/// Returns the config file, if it doesn't exist, an empty config is created
pub fn get_config_file() -> FResult<File> {
	let config_file_path = get_config_file_path();

	// If config file does not exist
	if !config_file_path.exists() {
		// Make sure config directory exists
		create_dir_all(config_file_path.parent().ok_or(FError::OptionError)?)?;

		// Create and the open config file
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.truncate(false)
			.create(true)
			.open(&config_file_path)?;

		// Write an empty config to the config file
		write_to_config(
			file,
			&Config {
				active_profile: 0,
				profiles: Vec::new(),
			},
		)?;
	}

	// Open and return the config file
	Ok(OpenOptions::new()
		.read(true)
		.write(true)
		.truncate(false)
		.create(false)
		.open(config_file_path)?)
}

/// Serialise and write `config` to `config_file`
pub fn write_to_config(mut config_file: File, config: &Config) -> FResult<()> {
	// Serialise config
	let contents = to_string_pretty(&config)?;

	config_file.set_len(0)?; // Truncate the file to 0
	config_file.seek(SeekFrom::Start(0))?; // Set header to beginning

	// Write config to file
	config_file.write_all(contents.as_bytes())?;

	Ok(())
}
