/*
 * This file contains convenience wrappers for the CLI
 */

use clap::{load_yaml, App};

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
    /// List mods and repos in the config
    List,
    /// Download and install the latest version of mods and repos in the config
    Upgrade,
}

/// Returns the subcommand (and its arguments) that needs to be executed
pub fn get_subcommand() -> SubCommand {
    // Load command definition from yaml file
    let yaml = load_yaml!("cli.yaml");
    let app = App::from_yaml(yaml).setting(clap::AppSettings::SubcommandRequiredElseHelp);
    let matches = app.get_matches();

    // Return enum accorsing to subcommand issued
    if let Some(_) = matches.subcommand_matches("list") {
        SubCommand::List
    } else if let Some(_) = matches.subcommand_matches("upgrade") {
        SubCommand::Upgrade
    } else if let Some(sub_matches) = matches.subcommand_matches("add") {
        SubCommand::Add {
            mod_id: sub_matches.value_of("MOD_ID").unwrap().into(), // Can unwrap because argument is required
        }
    } else if let Some(sub_matches) = matches.subcommand_matches("add-repo") {
        SubCommand::AddRepo {
            // Can unwrap because arguments are required
            owner: sub_matches.value_of("OWNER").unwrap().into(),
            name: sub_matches.value_of("REPO").unwrap().into(),
        }
    } else {
        panic!("Unknown subcommand");
    }
}
