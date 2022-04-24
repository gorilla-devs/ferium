use crate::subcommands::profile::pick_mod_loader;
use anyhow::Result;
use dialoguer::{Input, Select};
use libium::{config, file_picker, misc};
use std::path::PathBuf;

pub async fn configure(
    profile: &mut config::structs::Profile,
    game_version: Option<String>,
    mod_loader: Option<config::structs::ModLoader>,
    name: Option<String>,
    output_dir: Option<PathBuf>,
) -> Result<()> {
    let mut interactive = true;

    if let Some(game_version) = game_version {
        profile.game_version = game_version;
        interactive = false;
    }
    if let Some(mod_loader) = mod_loader {
        profile.mod_loader = mod_loader;
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

        loop {
            let selection = Select::with_theme(&*crate::THEME)
                .with_prompt("Which setting would you like to change")
                .items(&items)
                .interact_opt()?;

            if let Some(index) = selection {
                match index {
                    0 => {
                        if let Some(dir) = file_picker::pick_folder(&profile.output_dir).await {
                            profile.output_dir = dir;
                        }
                    },
                    1 => {
                        // Let user pick mc version from latest 10 versions
                        let mut versions = misc::get_major_mc_versions(10).await?;
                        let index = Select::with_theme(&*crate::THEME)
                            .with_prompt("Select a Minecraft version")
                            .items(&versions)
                            .default(0)
                            .interact_opt()?;
                        if let Some(i) = index {
                            profile.game_version = versions.swap_remove(i);
                        }
                    },
                    2 => {
                        // Let user pick mod loader
                        profile.mod_loader = pick_mod_loader(Some(&profile.mod_loader))?;
                    },
                    3 => {
                        let name = Input::with_theme(&*crate::THEME)
                            .with_prompt("Change the profile's name")
                            .default(profile.name.clone())
                            .interact_text()?;
                        profile.name = name;
                    },
                    4 => break,
                    _ => unreachable!(),
                }
                println!();
            } else {
                break;
            }
        }
    }

    Ok(())
}
