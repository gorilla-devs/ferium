use colored::Colorize as _;
use libium::config::structs::{Modpack, ModpackIdentifier};

pub fn info(modpack: &Modpack, active: bool) {
    println!(
        "{}{}
        \r  Output directory:  {}
        \r  Identifier:        {}
        \r  Install Overrides: {}\n",
        modpack.name.bold(),
        if active { " *" } else { "" },
        modpack.output_dir.display().to_string().blue().underline(),
        match &modpack.identifier {
            ModpackIdentifier::CurseForgeModpack(id) =>
                format!("{:10} {}", "CurseForge".red(), id.to_string().dimmed()),
            ModpackIdentifier::ModrinthModpack(id) =>
                format!("{:10} {}", "Modrinth".green(), id.dimmed()),
        },
        modpack.install_overrides
    );
}
