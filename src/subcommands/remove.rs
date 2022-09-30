use crate::THEME;
use anyhow::{bail, Result};
use dialoguer::MultiSelect;
use libium::config::structs::Profile;

/// Display a list of mods and repos in the profile to select from and remove
/// selected ones
pub fn remove(profile: &mut Profile, mod_names: Vec<String>) -> Result<()> {
    let names = profile
        .mods
        .iter()
        .map(|mod_| &mod_.name)
        .collect::<Vec<_>>();
    let mut items_to_remove = if mod_names.is_empty() {
        match MultiSelect::with_theme(&*THEME)
            .with_prompt("Select mods to remove")
            .items(&names)
            .interact_opt()?
        {
            Some(items_to_remove) => items_to_remove,
            None => return Ok(()), // Exit if the user cancelled
        }
    } else {
        let mut items_to_remove = Vec::new();
        for mod_name in mod_names {
            if let Some(index) = names
                .iter()
                .position(|name| name.to_lowercase() == mod_name.to_lowercase())
            {
                items_to_remove.push(index);
            } else {
                bail!("A mod called {} is not present in this profile", mod_name);
            }
        }
        items_to_remove
    };

    // Sort the indices in ascending order to fix moving indices during removal
    items_to_remove.sort_unstable();
    items_to_remove.reverse();
    for index in items_to_remove {
        profile.mods.swap_remove(index);
    }

    Ok(())
}
