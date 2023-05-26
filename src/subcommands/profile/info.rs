use colored::Colorize;
use libium::config::structs::Profile;

pub fn info(profile: &Profile, show_active_indicator: bool) -> String {
    return format!(
        "{}{}
        \r  Output directory:  {}
        \r  Minecraft Version: {}
        \r  Mod Loader:        {}
        \r  Mods:              {}",
        profile.name.bold(),
        if show_active_indicator { " (active)".green().bold() } else { "".normal() },
        profile.output_dir.display().to_string().blue().underline(),
        profile.game_version.green(),
        format!("{:?}", profile.mod_loader).purple(),
        profile.mods.len().to_string().yellow(),
    );
}
