use colored::Colorize;
use libium::config::structs::Config;

pub fn list(config: &Config) {
    for (i, profile) in config.profiles.iter().enumerate() {
        println!(
            "{}{}
        \r  Output directory:   {}
        \r  Minecraft Version:  {}
        \r  Mod Loader:         {}
        \r  Mods:               {}\n",
            profile.name.bold(),
            if i == config.active_profile { " *" } else { "" },
            profile.output_dir.display().to_string().blue().underline(),
            profile.game_version.green(),
            format!("{:?}", profile.mod_loader).purple(),
            profile.mods.len().to_string().yellow(),
        );
    }
}
