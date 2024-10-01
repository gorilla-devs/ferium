use super::{check_output_directory, pick_minecraft_versions, pick_mod_loader};
use anyhow::{Context as _, Result};
use inquire::{Select, Text};
use libium::{
    config::filters::ProfileParameters as _,
    config::structs::{ModLoader, Profile},
    file_picker::pick_folder,
};
use std::path::PathBuf;

pub async fn configure(
    profile: &mut Profile,
    game_versions: Vec<String>,
    mod_loaders: Vec<ModLoader>,
    name: Option<String>,
    output_dir: Option<PathBuf>,
) -> Result<()> {
    let mut interactive = true;

    if !game_versions.is_empty() {
        *profile
            .filters
            .game_versions_mut()
            .context("Active profile does not filter by game version")? = game_versions;

        interactive = false;
    }
    if !mod_loaders.is_empty() {
        *profile
            .filters
            .mod_loaders_mut()
            .context("Active profile does not filter mod loader")? = mod_loaders;

        interactive = false;
    }
    if let Some(name) = name {
        profile.name = name;
        interactive = false;
    }
    if let Some(output_dir) = output_dir {
        profile.output_dir = output_dir;
        interactive = false;
    }

    if interactive {
        let items = vec![
            // Show a file dialog
            "Mods output directory",
            // Show a picker of Minecraft versions to select from
            "Minecraft version",
            // Show a picker to change mod loader
            "Mod loader",
            // Show a dialog to change name
            "Profile Name",
            // Quit the configuration
            "Quit",
        ];

        while let Ok(selection) =
            Select::new("Which setting would you like to change", items.clone()).raw_prompt()
        {
            match selection.index {
                0 => {
                    if let Some(dir) = pick_folder(
                        &profile.output_dir,
                        "Pick an output directory",
                        "Output Directory",
                    )? {
                        check_output_directory(&dir).await?;
                        profile.output_dir = dir;
                    }
                }
                1 => {
                    let Some(versions) = profile.filters.game_versions_mut() else {
                        println!("Active profile does not filter by game version");
                        continue;
                    };

                    if let Ok(selection) = pick_minecraft_versions(versions).await {
                        *versions = selection;
                    }
                }
                2 => {
                    let Some(loaders) = profile.filters.mod_loaders_mut() else {
                        println!("Active profile does not filter mod loader");
                        continue;
                    };

                    if let Ok(selection) = pick_mod_loader(loaders.first()) {
                        *loaders = match selection {
                            ModLoader::Quilt => vec![ModLoader::Quilt, ModLoader::Fabric],
                            loader => vec![loader],
                        }
                    }
                }
                3 => {
                    if let Ok(new_name) = Text::new("Change the profile's name")
                        .with_default(&profile.name)
                        .prompt()
                    {
                        profile.name = new_name;
                    } else {
                        continue;
                    }
                }
                4 => break,
                _ => unreachable!(),
            }
            println!();
        }
    }

    Ok(())
}
