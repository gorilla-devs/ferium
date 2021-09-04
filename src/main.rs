mod labrinth;
mod util;

use labrinth::{funcs::*, structs::*};
use std::fs::OpenOptions;
use std::io::Write;
use util::*;

fn main() {
    let mut config_file = get_config_file();
    let config = get_config(&mut config_file);

    if config.mod_slugs.is_empty() {
        panic!(
            "No mods in config file! Fill up the config file's ({}) `mod_slugs` list with mod_ids",
            "~/.ferium/config.json"
        )
    }

    for mod_slug in config.mod_slugs {
        let mod_: Mod = get_mod(&mod_slug);
        println!("Downloading {}", mod_.title);

        print("  [1] Getting version information... ");
        let versions: Vec<Version> = get_versions(&mod_.id);
        println!("✓");

        let latest_version = &versions[0];

        let mut mod_jar = match OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!("{}{}.jar", config.output_dir, mod_.title))
        {
            Ok(file) => file,
            Err(e) => panic!("Could not open file due to {}", e),
        };

        print("  [2] Downloading mod file... ");
        let contents = download_version(latest_version);
        println!("✓");

        match mod_jar.write_all(&contents) {
            Ok(_) => (),
            Err(e) => panic!("File write failed due to {}", e),
        }
        println!("");
    }
}
