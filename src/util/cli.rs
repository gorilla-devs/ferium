//! Contains convenience wrappers for argument parsing using Clap

use crate::ferium_error::{FError, FResult};
use clap::{crate_version, load_yaml, App, AppSettings};

pub enum SubCommand {
	/// Check and add `mod_id` to profile
	Add {
		/// ID of the mod to add
		mod_id: String,
	},
	/// Check and add `owner` and `name` to profile
	AddRepo {
		/// Username of the owner of the repository to add
		owner: String,
		/// Name of the repository to add
		name: String,
	},
	/// Prompt user about which setting to configure, and let them change that setting
	Config,
	/// Create a new profile and add it to config
	Create,
	/// List mods and repos in the profile. Print more information if set to true (verbosity)
	List(bool),
	/// Remove one or more mods in the profile
	Remove,
	/// Switch to a different profile
	Switch,
	/// Download and install the latest version of mods and repos in the profile
	Upgrade,
}

/// Returns the subcommand (and its arguments) that needs to be executed
pub fn get_subcommand() -> FResult<SubCommand> {
	// Load command definition from yaml file
	let yaml = load_yaml!("cli.yaml");
	let app = App::from(yaml)
		.setting(AppSettings::SubcommandRequiredElseHelp)
		.version(crate_version!());
	let matches = app.get_matches();

	// Return enum according to subcommand issued
	if let Some(sub_matches) = matches.subcommand_matches("list") {
		Ok(SubCommand::List(sub_matches.is_present("verbose")))
	} else if matches.subcommand_matches("upgrade").is_some() {
		Ok(SubCommand::Upgrade)
	} else if let Some(sub_matches) = matches.subcommand_matches("add") {
		Ok(SubCommand::Add {
			// Can 'unwrap' because argument is required
			mod_id: sub_matches
				.value_of("MOD_ID")
				.ok_or(FError::OptionError)?
				.into(),
		})
	} else if let Some(sub_matches) = matches.subcommand_matches("add-repo") {
		Ok(SubCommand::AddRepo {
			// Can unwrap because arguments are required
			owner: sub_matches
				.value_of("OWNER")
				.ok_or(FError::OptionError)?
				.into(),
			name: sub_matches
				.value_of("REPO")
				.ok_or(FError::OptionError)?
				.into(),
		})
	} else if matches.subcommand_matches("remove").is_some() {
		Ok(SubCommand::Remove)
	} else if matches.subcommand_matches("config").is_some() {
		Ok(SubCommand::Config)
	} else if matches.subcommand_matches("create").is_some() {
		Ok(SubCommand::Create)
	} else if matches.subcommand_matches("switch").is_some() {
		Ok(SubCommand::Switch)
	} else {
		Err(FError::Quit {
			message: "Unknown subcommand".into(),
		})
	}
}
