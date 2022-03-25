//! Contains convenience wrappers for argument parsing using Clap
#![deny(missing_docs)] // All commands must have help/about statements

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(propagate_version = true)]
#[clap(subcommand_required = true)]
pub struct Ferium {
	#[clap(subcommand)]
	pub subcommand: SubCommands,
	#[clap(long)]
	#[clap(hide = true)]
	#[clap(help("Nur zum testen"))]
	pub config_file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum SubCommands {
	#[clap(about("Add a Modrinth mod to the profile"))]
	AddModrinth {
		#[clap(help("The project ID is specified as '</> PROJECT ID' in the right sidebar of the mod's Modrith page\nYou can also use the project slug for this"))]
		project_id: String,
	},
	#[clap(about("Add a GitHub repository to the profile"))]
	AddGithub {
		#[clap(help("The repository owner's username"))]
		owner: String,
		#[clap(help("The name of the repository"))]
		name: String,
	},
	#[clap(about("Add a CurseForge mod to the profile"))]
	AddCurseforge {
		#[clap(help("The project ID is specified as 'Project ID' in the 'About Project' sidebar of the mod's CurseForge page"))]
		project_id: i32,
	},
	#[clap(about("List all the mods in the profile, and with some their metadata if verbose"))]
	List {
		#[clap(long, short)]
		#[clap(help("Show information about the mod"))]
		verbose: bool,
	},
	#[clap(subcommand_required = true)]
	#[clap(about("Create, configure, or remove the current profile"))]
	Profile {
		#[clap(subcommand)]
		subcommand: ProfileSubCommands,
	},
	#[clap(about("Remove a mod or repository from the profile\nOptionally, provide a list of names of the mods to remove"))]
	Remove {
		#[clap(long)]
		#[clap(name("mod-name"))]
		#[clap(help("A case-insensitive name of a mod to remove\nYou can repeat this option to remove multiple mods\nIf one or more of the mod names provided does not exist, the program will error out without changing anything in the config"))]
		mod_names: Option<Vec<String>>,
	},
	#[clap(about("Switch between different profiles\nOptionally, provide the name of the profile to switch to"))]
	Switch {
		#[clap(long)]
		#[clap(help("The name of the profile to switch to"))]
		profile_name: Option<String>,
	},
	#[clap(about("Download and install the latest version of the mods specified"))]
	Upgrade {
		#[clap(long)]
		#[clap(help(
			"Do not check for the full game version, only check for the major and minor versions
			\rSome Minecraft versions (e.g. 1.18 & 1.18.1) are compatible with each other,
			\rthis option allows you to use older, but still compatible, versions of a mod that might not have yet updated to the latest version"
		))]
		no_patch_check: bool,
	},
}

#[derive(Subcommand)]
pub enum ProfileSubCommands {
	#[clap(about(
		"Configure the current profile's Minecraft version, mod loader, and output directory\nOptionally, provide setting(s) to change as option(s)"
	))]
	Configure {
		#[clap(long)]
		#[clap(help("The Minecraft version to check compatibility for"))]
		game_version: Option<String>,
		#[clap(long)]
		#[clap(arg_enum)]
		#[clap(help("The mod loader to check compatibility for"))]
		mod_loader: Option<libium::config::structs::ModLoaders>,
		#[clap(long)]
		#[clap(help("The name of the profile"))]
		name: Option<String>,
		#[clap(long)]
		#[clap(help("The directory to output mods to"))]
		output_dir: Option<PathBuf>,
	},
	#[clap(about("Create a new profile\nOptionally, provide ALL the options to create the profile without the UI"))]
	Create {
		#[clap(long)]
		#[clap(help("The Minecraft version to check compatibility for"))]
		game_version: Option<String>,
		#[clap(long)]
		#[clap(help("Do not check whether the game version exists or not"))]
		force_game_version: bool,
		#[clap(long)]
		#[clap(arg_enum)]
		#[clap(help("The mod loader to check compatibility for"))]
		mod_loader: Option<libium::config::structs::ModLoaders>,
		#[clap(long)]
		#[clap(help("The name of the profile"))]
		name: Option<String>,
		#[clap(long)]
		#[clap(help("The directory to output mods to"))]
		output_dir: Option<PathBuf>,
	},
	#[clap(about("Delete a profile\nOptionally, provide the name of the profile to delete\nAfter deletion, the first profile will be selected"))]
	Delete {
		#[clap(long)]
		#[clap(help("The name of the profile to delete"))]
		profile_name: Option<String>,
	},
	#[clap(about("List all the profiles with their data"))]
	List,
}
