use colored::Colorize;
use libium::config::structs::{Modpack, ModpackIdentifier};

pub fn info(modpack: &Modpack, show_active_indicator: bool) -> String {
    format!(
        "\
{}
  Output directory:  {}
  Identifier:        {}
  Install Overrides: {}",
  if show_active_indicator {
      format!("{} (active)", modpack.name)
                .bright_yellow()
                .bold()
                .underline()
        } else {
            modpack.name.bold()
        },
        modpack.output_dir.display().to_string().blue().underline(),
        match &modpack.identifier {
            ModpackIdentifier::CurseForgeModpack(id) =>
                format!("{:10} {}", "CurseForge".red(), id.to_string().dimmed()),
            ModpackIdentifier::ModrinthModpack(id) =>
                format!("{:10} {}", "Modrinth".green(), id.dimmed()),
        },
        modpack.install_overrides,
    )
}
