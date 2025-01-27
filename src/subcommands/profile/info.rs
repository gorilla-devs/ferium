use colored::Colorize;
use libium::{
    config::{filters::ProfileParameters as _, structs::{Profile, ProfileItem}},
    iter_ext::IterExt as _,
};

pub fn info(profile_item: &ProfileItem, profile: &Profile, active: bool) {
    println!(
        "{}{}
        \r  Profile Path:       {}
        \r  Output directory:   {}{}{}
        \r  Mods:               {}\n",
        profile_item.name.bold(),
        if active { " *" } else { "" },
        profile_item.path.display().to_string().blue().underline(),
        profile_item.output_dir.display().to_string().blue().underline(),
        profile
            .filters
            .game_versions()
            .map(|v| format!(
                "\n  Minecraft Version:  {}",
                v.iter()
                    .map(AsRef::as_ref)
                    .map(Colorize::green)
                    .display(", ")
            ))
            .unwrap_or_default(),
        profile
            .filters
            .mod_loader()
            .map(|l| format!("\n  Mod Loader:         {}", l.to_string().purple()))
            .unwrap_or_default(),
        profile.mods.len().to_string().yellow(),
    );
}
