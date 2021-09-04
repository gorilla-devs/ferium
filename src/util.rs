use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};
use shellexpand::tilde;
use std::env::consts::OS;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{stdout, Read, Write};
use std::path::Path;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    /// The directory to download files to
    pub output_dir: String,
    /// A list of mod slugs specifiying the mods to download
    pub mod_slugs: Vec<String>,
}

/// Reads from `config_file` and returns a deserialised config
pub fn get_config(config_file: &mut File) -> Config {
    // Read file contents
    let mut contents = String::new();
    match config_file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(e) => panic!("Could not read file due to {}", e),
    }

    // Try deserialising contents and return if successful
    match from_str(&contents) {
        Ok(config) => config,
        Err(e) => panic!(
            "Config file deserialisation failed due to {}. Check that the formatting is correct!",
            e
        ),
    }
}

/// Returns the config file. If not found, a config file will be created and written to with default values
pub fn get_config_file() -> File {
    // Directory where configs are stored
    let config_file_dir = tilde("~/.ferium/").to_string();
    // Config file path
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
            Err(e) => panic!("Could not open config file due to {}", e),
        }

    // If config file does not exist
    } else {
        // Create config directory
        match create_dir_all(config_file_dir) {
            Ok(_) => (),
            Err(e) => panic!("Could not create config directory due to {}", e)
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
                    Config {
                        output_dir: get_mods_dir().into(),
                        mod_slugs: Vec::new(),
                    },
                );
                file
            }
            Err(e) => panic!("Could not create/open config file due to {}", e),
        }
    }
}

/// Serialise and write `config` to `config_file`
pub fn write_to_config(config_file: &mut File, config: Config) {
    // Serialise config
    let contents = match to_string_pretty(&config) {
        Ok(contents) => contents,
        Err(e) => panic!("Could not serialise JSON due to {}", e),
    };

    // Truncate file and write config
    config_file.set_len(0).unwrap();
    match config_file.write_all(contents.as_bytes()) {
        Ok(_) => (),
        Err(e) => panic!("Could not write to config file due to {}", e),
    }
}

/// Returns the default directory where mods are stored
fn get_mods_dir() -> std::borrow::Cow<'static, str> {
    if OS == "macos" {
        tilde("~/Library/ApplicationSupport/minecraft/mods/")
    } else if OS == "linux" {
        tilde("~/.minecraft/mods/")
    } else if OS == "windows" {
        tilde("~\\AppData\\Roaming\\.minecraft\\mods\\")
    } else {
        panic!("Not running on a device capable of running Minecraft Java Edition!")
    }
}

/// Run `print` macro and flush stdout to make results immediately appear
pub fn print(msg: &str) {
    print!("{}", msg);
    stdout().flush().unwrap();
}
