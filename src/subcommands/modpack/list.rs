use itertools::Itertools;
use libium::config::structs::Config;
use crate::subcommands::modpack::info;


pub fn list(config: &Config) {
    println!(
        "{}",
        config
            .modpacks
            .iter()
            .enumerate()
            .map(|(i, profile)| info::info(profile, i == config.active_profile))
            .join("\n\n")
    );
}
