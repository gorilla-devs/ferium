//! This file contains convenience wrappers for the CLI

use crate::ferium_error::*;
use clap::{crate_version, load_yaml, App, AppSettings};

/// Enum for subcommands
pub enum SubCommand {
    /// Check and add `mod_id` to `Config.mod_slugs`
    Add {
        /// ID of the mod to add
        mod_id: String,
    },
    /// Check and add `owner` and `name` to `Config.repos`
    AddRepo {
        /// Username of the owner of the repository to add
        owner: String,
        /// Name of the repository to add
        name: String,
    },
    /// Show user which setting to configure, and let them change that setting
    Config,
    /// List mods and repos in the config
    List,
    /// Remove one or more mods in the config
    Remove,
    /// Download and install the latest version of mods and repos in the config
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
    if matches.subcommand_matches("list").is_some() {
        Ok(SubCommand::List)
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
    } else {
        Err(FError::Quit {
            message: "Unknown subcommand".into(),
        })
    }
}
