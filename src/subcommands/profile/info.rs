use colored::Colorize;
use libium::config::structs::Profile;

pub fn info(profile: &Profile, active: bool) {
    println!(
        "{}{}
        \r  Output directory:   {}
        \r  Minecraft Version:  {}
        \r  Mod Loader:         {}
        \r  Mods:               {}\n",
        profile.name.bold(),
        if active { " *" } else { "" },
        profile.output_dir.display().to_string().blue().underline(),
        profile.game_version.green(),
        format!("{:?}", profile.mod_loader).purple(),
        profile.mods.len().to_string().yellow(),
    );
}
