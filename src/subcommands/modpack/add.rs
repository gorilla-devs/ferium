use crate::{subcommands::modpack::check_output_directory, THEME, TICK};
use anyhow::{anyhow, Result};
use dialoguer::Confirm;
use ferinth::Ferinth;
use furse::Furse;
use libium::{config::structs::Config, file_picker, misc::get_minecraft_dir, modpack::add};
use std::{path::PathBuf, sync::Arc};

pub async fn curseforge(
    curseforge: Arc<Furse>,
    config: &mut Config,
    project_id: i32,
    output_dir: &Option<PathBuf>,
    install_overrides: Option<bool>,
) -> Result<()> {
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
    eprint!("Adding modpack... ");
    let project = add::curseforge(
        curseforge,
        config,
        project_id,
        output_dir,
        install_overrides,
    )
    .await?;
    // Make added modpack active
    config.active_modpack = config.modpacks.len() - 1;
    println!("{} ({})", *TICK, project.name);
    Ok(())
}

pub async fn modrinth(
    modrinth: Arc<Ferinth>,
    config: &mut Config,
    project_id: &str,
    output_dir: &Option<PathBuf>,
    install_overrides: Option<bool>,
) -> Result<()> {
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
    eprint!("Adding modpack... ");
    let project =
        add::modrinth(modrinth, config, project_id, output_dir, install_overrides).await?;
    // Make added modpack active
    config.active_modpack = config.modpacks.len() - 1;
    println!("{} ({})", *TICK, project.title);
    Ok(())
}
