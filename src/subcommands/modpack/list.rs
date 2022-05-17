use colored::Colorize;
use libium::config::structs::{Config, ModpackIdentifier};

pub fn list(config: &Config) {
    for (i, modpack) in config.modpacks.iter().enumerate() {
        println!(
            "{}{}
        \r  Output directory:   {}
        \r  Identifier:         {}\n",
            modpack.name.bold(),
            if i == config.active_modpack { " *" } else { "" },
            modpack.output_dir.display().to_string().blue().underline(),
            match &modpack.identifier {
                ModpackIdentifier::CurseForgeModpack(id) =>
                    format!("{:10} {}", "CurseForge".red(), id.to_string().dimmed()),
                ModpackIdentifier::ModrinthModpack(id) =>
                    format!("{:10} {}", "Modrinth".green(), id.dimmed()),
            },
        );
    }
}
