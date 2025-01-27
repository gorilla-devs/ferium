use super::check_output_directory;
use crate::file_picker::pick_file;
use anyhow::Result;
use colored::Colorize as _;
use inquire::Confirm;
use libium::config::structs::Modpack;
use std::path::PathBuf;

pub fn configure(
    modpack: &mut Modpack,
    output_dir: Option<PathBuf>,
    install_overrides: Option<bool>,
    no_gui_mode: Option<bool>,
) -> Result<()> {
    match output_dir {
        Some(output_dir) => {
            check_output_directory(&output_dir, no_gui_mode)?;
            modpack.output_dir = output_dir;
        }
        None => {
            if let Some(dir) = pick_file(
                &modpack.output_dir,
                "Pick an output directory",
                "Output Directory",
                true,
                no_gui_mode,
            )? {
                check_output_directory(&dir, no_gui_mode)?;
                modpack.output_dir = dir;
            }
        }
    }
    modpack.install_overrides = if let Some(install_overrides) = install_overrides {
        install_overrides
    } else {
        let install_overrides = Confirm::new("Should overrides be installed?")
            .with_default(modpack.install_overrides)
            .prompt()
            .unwrap_or(modpack.install_overrides);
        if install_overrides {
            println!(
                "{}",
                "WARNING: Configs in your output directory may be overwritten by modpack overrides"
                    .yellow()
                    .bold()
            );
        }
        install_overrides
    };
    Ok(())
}
