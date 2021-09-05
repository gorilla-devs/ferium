/*
 * This file contains convenience wrappers for the CLI
 */

use clap::{load_yaml, App};

/// Enum for all the commands
pub enum SubCommand {
    /// Add `mod_id` to config
    Add { mod_id: String },
    /// Display help page
    Help,
    /// List mods in config
    List,
    /// Download and install latest version of mods in config
    Upgrade,
}

/// Returns the subcommand (and its arguments) that needs to be executed
pub fn get_subcommand() -> SubCommand {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(_) = matches.subcommand_matches("list") {
        SubCommand::List
    } else if let Some(_) = matches.subcommand_matches("upgrade") {
        SubCommand::Upgrade
    } else if let Some(_) = matches.subcommand_matches("help") {
        SubCommand::Help
    } else if let Some(sub_matches) = matches.subcommand_matches("add") {
        SubCommand::Add {
            mod_id: sub_matches.value_of("MOD_ID").unwrap().into(), // Can unwrap because argument is required
        }
    } else {
        SubCommand::Help
    }
}
