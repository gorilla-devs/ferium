use colored::Colorize;
use libium::config::structs::Profile;

pub fn info(profile: &Profile, show_active_indicator: bool) -> String {
    return format!(
        "\
{}
  Output directory:  {}
  Minecraft Version: {}
  Mod Loader:        {}
  Mods:              {}",
        if show_active_indicator {
            format!("{} (active)", profile.name).bright_yellow().bold().underline()
        } else {
            profile.name.bold()
        },
        profile.output_dir.display().to_string().blue().underline(),
        profile.game_version.green(),
        format!("{:?}", profile.mod_loader).purple(),
        profile.mods.len().to_string().yellow(),
    );
}

pub fn info_md(profile: &Profile) -> String {
    return format!(
        "\
# {}

|                   |                 |
|-------------------|:----------------|
| Minecraft Version | _{}_            |
| Mod Loader        | {}              |
| Mods              | {}              |",
        profile.name,
        profile.game_version,
        format!("{:?}", profile.mod_loader),
        profile.mods.len().to_string(),
    );
}
