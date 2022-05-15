use colored::Colorize;
use libium::config::structs::{Config, ModpackIdentifier};

pub fn list(config: &Config) {
    for (i, modpack) in config.modpacks.iter().enumerate() {
        println!(
            "{}{}
        \r  Output directory:   {}
        \r  Identifier:         {:10} {}\n",
            modpack.name.bold(),
            if i == config.active_modpack { " *" } else { "" },
            modpack.output_dir.display().to_string().blue().underline(),
            match &modpack.identifier {
                ModpackIdentifier::CurseForgeModpack(_) => "CurseForge".red(),
                // ModpackIdentifier::ModrinthModpack(_) => "Modrinth".green(),
            },
            match &modpack.identifier {
                ModpackIdentifier::CurseForgeModpack(id) => id.to_string(),
                // ModpackIdentifier::ModrinthModpack(id) => id.into(),
            }
            .dimmed(),
        );
    }
}
