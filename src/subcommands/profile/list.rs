use libium::config;

pub fn list(config: &config::structs::Config) {
    for (i, profile) in config.profiles.iter().enumerate() {
        println!(
            "{}{}
        \r  Output directory:   {}
        \r  Minecraft Version:  {}
        \r  Mod Loader:         {}
        \r  Mods:               {}\n",
            profile.name,
            if i == config.active_profile { " *" } else { "" },
            profile.output_dir.display(),
            profile.game_version,
            profile.mod_loader,
            profile.mods.len(),
        );
    }
}
