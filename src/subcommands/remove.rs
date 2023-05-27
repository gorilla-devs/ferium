use crate::{get_oneline_mod_info, THEME};
use anyhow::{bail, Result};
use colored::Colorize;
use dialoguer::MultiSelect;
use libium::config::structs::{ModIdentifier, Profile};

/// If `to_remove` is empty, display a list of projects in the profile to select from and remove selected ones
///
/// Else, search the given strings with the projects' name and IDs and remove them
pub fn remove(profile: &mut Profile, to_remove: Vec<String>) -> Result<()> {
    let mut indices_to_remove = if to_remove.is_empty() {
        match MultiSelect::with_theme(&*THEME)
            .with_prompt("Select mods to remove")
            .items(
                &profile
                    .mods
                    .iter()
                    .map(|mod_| &mod_.name)
                    .collect::<Vec<_>>(),
            )
            .report(false)
            .interact_opt()?
        {
            Some(items_to_remove) => items_to_remove,
            None => return Ok(()), // Exit if the user cancelled
        }
    } else {
        let mut items_to_remove = Vec::new();
        for to_remove in to_remove {
            if let Some(index) = profile.mods.iter().position(|mod_| {
                mod_.name.to_lowercase() == to_remove.to_lowercase()
                    || match &mod_.identifier {
                        ModIdentifier::CurseForgeProject(id) => id.to_string() == to_remove,
                        ModIdentifier::ModrinthProject(id) => id == &to_remove,
                        ModIdentifier::GitHubRepository((owner, name)) => {
                            format!("{owner}/{name}").to_lowercase() == to_remove.to_lowercase()
                        }
                    }
            }) {
                items_to_remove.push(index);
            } else {
                bail!("A mod with ID or name {to_remove} is not present in this profile");
            }
        }
        items_to_remove
    };

    // Sort the indices in ascending order to fix moving indices during removal
    indices_to_remove.sort_unstable();
    indices_to_remove.reverse();

    let mut removed = Vec::new();
    for index in indices_to_remove {
        removed.push(profile.mods.swap_remove(index));
    }

    println!("{}", "Removed:".red());
    for mod_ in removed {
        println!("  {}", get_oneline_mod_info(&mod_));
    }

    Ok(())
}
