/*
 * This file contains convenience wrappers for configurations and general JSON stuff
 */

use super::wrappers::get_mods_dir;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use shellexpand::tilde;
use std::cmp::PartialEq;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

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

/// Returns the config file. If not found, a config file will be created and written to with default values
pub fn get_config_file() -> File {
    let home = tilde("~");
    let home = Path::new(home.as_ref());
    // Config file's path
    let config_file_path = home.join(".ferium").join("config.json");

    // If config file exists
    if config_file_path.exists() {
        // Open and return it
        OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(false)
            .open(config_file_path)
            .unwrap()

    // If config file does not exist
    } else {
        // Make sure config directory exists
        create_dir_all(config_file_path.parent().unwrap()).unwrap();

        // Create and open config file
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(config_file_path)
            .unwrap();
        // Write to config file with defaults
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
}

/// Serialise and write `config` to `config_file`
pub fn write_to_config(config_file: &mut File, config: &Config) {
    // Serialise config
    let contents = to_string_pretty(&config).unwrap();

    // Truncate and rewind file
    config_file.set_len(0).unwrap();
    config_file
        .seek(SeekFrom::Start(0))
        .expect("Could not rewind config file");

    // Write config to file
    config_file.write_all(contents.as_bytes()).unwrap()
}
