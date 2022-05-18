use crate::{subcommands::modpack::check_output_directory, THEME, TICK};
use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::Confirm;
use ferinth::Ferinth;
use furse::Furse;
use itertools::Itertools;
use libium::{
    config::structs::{Config, Modpack, ModpackIdentifier},
    file_picker,
    misc::get_minecraft_dir,
    modpack::add,
};
use std::{path::PathBuf, sync::Arc};

pub async fn curseforge(
    curseforge: Arc<Furse>,
    config: &mut Config,
    project_id: i32,
    output_dir: &Option<PathBuf>,
    install_overrides: Option<bool>,
) -> Result<()> {
    eprint!("Checking modpack... ");
    let project = add::curseforge(curseforge, config, project_id).await?;
    println!("{} ({})", *TICK, project.name);
    println!("Where should the modpack be installed to?");
    let output_dir = match output_dir {
        Some(some) => some.clone(),
        None => file_picker::pick_folder(&get_minecraft_dir(), "Pick an output directory")
            .await
            .ok_or_else(|| anyhow!("Please pick an output directory"))?,
    };
    check_output_directory(&output_dir).await?;
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
        );
    }
    config.modpacks.push(Modpack {
        name: project.name.clone(),
        identifier: ModpackIdentifier::CurseForgeModpack(project.id),
        output_dir,
        install_overrides,
    });
    // Make added modpack active
    config.active_modpack = config.modpacks.len() - 1;
    Ok(())
}

pub async fn modrinth(
    modrinth: Arc<Ferinth>,
    config: &mut Config,
    project_id: &str,
    output_dir: &Option<PathBuf>,
    install_overrides: Option<bool>,
) -> Result<()> {
    eprint!("Checking modpack... ");
    let project = add::modrinth(modrinth, config, project_id).await?;
    println!("{} ({})", *TICK, project.title);
    println!("Where should the modpack be installed to?");
    let output_dir = match output_dir {
        Some(some) => some.clone(),
        None => file_picker::pick_folder(&get_minecraft_dir(), "Pick an output directory")
            .await
            .ok_or_else(|| anyhow!("Please pick an output directory"))?,
    };
    check_output_directory(&output_dir).await?;
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
        );
    }
    if let Some(donation_urls) = project.donation_urls {
        println!(
            "Consider supporting the mod creator on {}",
            donation_urls
                .iter()
                .map(|this| format!("{} ({})", this.platform.bold(), this.url.blue().underline()))
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
