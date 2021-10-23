//! This file contains convenience wrappers for the CLI

use crate::ferium_error::*;
use clap::{crate_version, load_yaml, App};

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
    /// Remove one or more mods in the config
    Remove,
    /// List mods and repos in the config
    List,
    /// Download and install the latest version of mods and repos in the config
    Upgrade,
}

/// Returns the subcommand (and its arguments) that needs to be executed
pub fn get_subcommand() -> FResult<SubCommand> {
    // Load command definition from yaml file
    let yaml = load_yaml!("cli.yaml");
    let app = App::from_yaml(yaml)
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .version(crate_version!());
    let matches = app.get_matches();

    // Return enum according to subcommand issued
    if let Some(_) = matches.subcommand_matches("list") {
        Ok(SubCommand::List)
    } else if let Some(_) = matches.subcommand_matches("upgrade") {
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
    } else if let Some(_) = matches.subcommand_matches("remove") {
        Ok(SubCommand::Remove)
    } else {
        Err(FError::Quit {
            message: "Unknown subcommand".into(),
        })
    }
}
