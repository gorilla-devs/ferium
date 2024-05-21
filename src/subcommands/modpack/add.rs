use super::check_output_directory;
use crate::{THEME, TICK};
use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::Confirm;
use ferinth::Ferinth;
use furse::Furse;
use itertools::Itertools;
use libium::{
    config::structs::{Config, Modpack, ModpackIdentifier},
    file_picker::pick_folder,
    get_minecraft_dir,
    modpack::add,
};
use std::path::PathBuf;

pub async fn curseforge(
    curseforge: &Furse,
    config: &mut Config,
    project_id: i32,
    output_dir: Option<PathBuf>,
    install_overrides: Option<bool>,
) -> Result<()> {
    eprint!("Checking modpack... ");
    let project = add::curseforge(curseforge, config, project_id).await?;
    println!("{} ({})", *TICK, project.name);
    println!("Where should the modpack be installed to?");
    let output_dir = match output_dir {
        Some(some) => some,
        None => pick_folder(
            get_minecraft_dir(),
            "Pick an output directory",
            "Output Directory",
        )?
        .ok_or_else(|| anyhow!("Please pick an output directory"))?,
    };
    check_output_directory(&output_dir)?;
    let install_overrides = match install_overrides {
        Some(some) => some,
        None => Confirm::with_theme(&*THEME)
            .default(true)
            .with_prompt("Should overrides be installed?")
            .interact()?,
    };
    if install_overrides {
        println!(
            "{}",
            "WARNING: Files in your output directory may be overwritten by modpack overrides"
                .yellow()
                .bold()
        );
    }
    config.modpacks.push(Modpack {
        name: project.name,
        identifier: ModpackIdentifier::CurseForgeModpack(project.id),
        output_dir,
        install_overrides,
    });
    // Make added modpack active
    config.active_modpack = config.modpacks.len() - 1;
    Ok(())
}

pub async fn modrinth(
    modrinth: &Ferinth,
    config: &mut Config,
    project_id: &str,
    output_dir: Option<PathBuf>,
    install_overrides: Option<bool>,
) -> Result<()> {
    eprint!("Checking modpack... ");
    let project = add::modrinth(modrinth, config, project_id).await?;
    println!("{} ({})", *TICK, project.title);
    println!("Where should the modpack be installed to?");
    let output_dir = match output_dir {
        Some(some) => some,
        None => pick_folder(
            get_minecraft_dir(),
            "Pick an output directory",
            "Output Directory",
        )?
        .ok_or_else(|| anyhow!("Please pick an output directory"))?,
    };
    check_output_directory(&output_dir)?;
    let install_overrides = match install_overrides {
        Some(some) => some,
        None => Confirm::with_theme(&*THEME)
            .default(true)
            .with_prompt("Should overrides be installed?")
            .interact()?,
    };
    if install_overrides {
        println!(
            "{}",
            "WARNING: Configs in your output directory may be overwritten by modpack overrides"
                .yellow()
                .bold()
        );
    }
    if !project.donation_urls.is_empty() {
        println!(
            "Consider supporting the mod creator on {}",
            project
                .donation_urls
                .iter()
                .map(|this| format!(
                    "{} ({})",
                    this.platform.bold(),
                    this.url.to_string().blue().underline()
                ))
                .format(", ")
        );
    }
    config.modpacks.push(Modpack {
        name: project.title,
        identifier: ModpackIdentifier::ModrinthModpack(project.id),
        output_dir,
        install_overrides,
    });
    // Make added modpack active
    config.active_modpack = config.modpacks.len() - 1;
    Ok(())
}
