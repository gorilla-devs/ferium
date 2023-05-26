use itertools::Itertools;
use libium::config::structs::Config;

use crate::subcommands::profile::info;

pub fn list(config: &Config) {
    println!(
        "{}",
        config
            .profiles
            .iter()
            .enumerate()
            .map(|(i, profile)| info(profile, i == config.active_profile))
            .join("\n\n")
    );
}
