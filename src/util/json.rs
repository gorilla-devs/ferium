/*
 * This file contains convenience wrappers for configurations and general JSON stuff
 */

use super::wrappers::get_mods_dir;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};
use shellexpand::tilde;
use std::cmp::PartialEq;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::process::exit;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    /// The directory to download mod JARs to
    pub output_dir: String,
    /// A list of mod slugs specifiying the mods to download
    pub mod_slugs: Vec<String>,
    /// A list of repositories of mods to download
    pub repos: Vec<Repo>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Repo {
    // Username of the owner of the repository
    pub owner: String,
    // Name of the repository
    pub name: String,
}

/// Reads from `config_file` and returns a deserialised config
pub fn get_config(config_file: &mut File) -> Config {
    // Read file contents
    let mut contents = String::new();
    match config_file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(e) => {
            println!("Could not read file due to {}", e);
            exit(120);
        }
    }

    // Try deserialising contents and return it if successful
    match from_str(&contents) {
        Ok(config) => config,
        Err(e) => {
            println!(
                "Config file deserialisation failed due to {}. Check that the formatting is correct!",
                e
            );
            exit(120);
        }
    }
}

/// Returns the config file. If not found, a config file will be created and written to with default values
pub fn get_config_file() -> File {
    // Directory where configs are stored
    let config_file_dir = tilde("~/.ferium/").to_string();
    // Config file's path
    let config_file_path = format!("{}{}", config_file_dir, "config.json");
    let config_file_path = Path::new(&config_file_path);

    // If config file exists
    if config_file_path.exists() {
        // Open and return it
        match OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(false)
            .open(config_file_path)
        {
            Ok(file) => file,
            Err(e) => {
                println!("Could not open config file due to {}", e);
                exit(120);
            }
        }

    // If config file does not exist
    } else {
        // Make sure config directory exists
        match create_dir_all(config_file_dir) {
            Ok(_) => (),
            Err(e) => {
                println!("Could not create config directory due to {}", e);
                exit(120);
            }
        }

        // Create and open config file
        match OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(config_file_path)
        {
            Ok(mut file) => {
                // Write default values to the config file
                write_to_config(
                    &mut file,
                    &Config {
                        output_dir: get_mods_dir().into(),
                        mod_slugs: Vec::new(),
                        repos: Vec::new(),
                    },
                );
                file
            }
            Err(e) => {
                println!("Could not create/open config file due to {}", e);
                exit(120);
            }
        }
    }
}

/// Serialise and write `config` to `config_file`
pub fn write_to_config(config_file: &mut File, config: &Config) {
    // Serialise config
    let contents = match to_string_pretty(&config) {
        Ok(contents) => contents,
        Err(e) => {
            println!("Could not serialise JSON due to {}", e);
            exit(122);
        }
    };

    // Truncate and rewind file, and write config
    config_file.set_len(0).unwrap();
    config_file
        .seek(SeekFrom::Start(0))
        .expect("Could not rewind config file");
    match config_file.write_all(contents.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Could not write to config file due to {}", e);
            exit(120);
        }
    }
}
