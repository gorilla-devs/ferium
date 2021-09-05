mod labrinth;
mod util;

use labrinth::funcs::*;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::process::exit;
use util::{cli::*, json::*, wrappers::print};

const HELP: &str = "Ferium is an open source and easy to use package manager for Minecraft mods (that are on https://modrinth.com).

USAGE
    ferium <command> [arguments]
    
COMMANDS
    list:       List all the mods configured
    help:       Show this help page
    upgrade:    Download and install the latest version of the mods specified
    add MOD_ID: Add a mod to the config.
                A mod's MOD_ID is specified as '</> PROJECT ID' in the right sidebar of the mod's Modrith page
    
EXAMPLES
    $ ferium upgrade        # Upgrades all the mods in your config
    $ ferium add AANobbMI   # Adds the Sodium mod to your config

ERROR CODES
    120: I/O error
    122: JSON error
    124: Server/HTTP error
    126: General error

FEEDBACK
    You can open an issue at https://github.com/theRookieCoder/ferium/issues/new.
    If you know how to and are willing to fix it, then you can make a pull request!
    
CONTRIBUTE
    Think you can improve Ferium? Well head on to https://github.com/theRookieCoder/ferium and you can start working on Ferium!";

fn main() {
    let mut config_file = get_config_file(); // Get config file
    let mut config = get_config(&mut config_file); // Get config from config file
    let command = get_subcommand(); // Get command information

    match command {
        SubCommand::Add { mod_id } => add_mod(mod_id, &mut config, &mut config_file),
        SubCommand::List => list(config),
        SubCommand::Upgrade => upgrade(config),
        SubCommand::Help => {
            println!("{}", HELP);
            exit(126);
        }
    };
}

/// Check if mod with `mod_id` exists, and if so, add that mod to `config_file`
fn add_mod(mod_id: String, config: &mut Config, config_file: &mut File) {
    if config.mod_slugs.contains(&mod_id) {
        println!("Mod already added to config!");
        exit(126);
    }

    print(&format!("Adding mod {}... ", mod_id));
    if let Some(mod_) = does_exist(&mod_id) {
        config.mod_slugs.push(mod_id);
        write_to_config(config_file, config);
        println!("✓ ({})", mod_.title);
    } else {
        println!("Mod with ID {} does not exist!", mod_id);
        exit(126);
    }
}

/// List all the mods in `config` and some of their metadata
fn list(config: Config) {
    // Check if empty and tell user to add mods
    if config.mod_slugs.is_empty() {
        println!("Your config file contains no mods! Run `ferium help` to see how to add mods.");
        exit(126);
    }

    for mod_slug in config.mod_slugs {
        // Get mod metadata
        let mod_ = get_mod(&mod_slug);
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
}

/// Download and install all mods in `config`
fn upgrade(config: Config) {
    // Check if empty and tell user to add mods
    if config.mod_slugs.is_empty() {
        println!("Your config file contains no mods! Run `ferium help` to see how to add mods.");
        exit(126);
    }

    for mod_slug in config.mod_slugs {
        // Get mod metadata
        let mod_ = get_mod(&mod_slug);
        println!("Downloading {}", mod_.title);

        // Get versions of the mod
        print("  [1] Getting version information... ");
        let versions = get_versions(&mod_.id);
        println!("✓");

        // Versions are arranged chronologically so first one is the latest
        let latest_version = &versions[0];

        // Open mod JAR file
        let mut mod_jar = match OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!("{}{}.jar", config.output_dir, mod_.title))
        {
            Ok(file) => file,
            Err(e) => {
                println!("Could not open file due to {}", e);
                exit(120)
            }
        };

        // Download file
        print(&format!("  [2] Downloading {}... ", latest_version.name));
        let contents = download_version(&latest_version);
        println!("✓");

        // Write download to JAR file
        match mod_jar.write_all(&contents) {
            Ok(_) => (),
            Err(e) => {
                println!("File write failed due to {}", e);
                exit(120);
            }
        }
        println!("");
    }
}
