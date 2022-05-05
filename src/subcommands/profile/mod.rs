mod configure;
mod create;
mod delete;
mod list;
pub use configure::configure;
pub use create::create;
pub use delete::delete;
pub use list::list;

use anyhow::{bail, Result};
use dialoguer::Select;
use libium::{
    config::{self, structs::ModLoader},
    misc,
};

fn pick_mod_loader(default: Option<&ModLoader>) -> std::io::Result<ModLoader> {
    let mut picker = Select::with_theme(&*crate::THEME);
    picker
        .with_prompt("Which mod loader do you use?")
        .items(&["Quilt", "Fabric", "Forge"]);
    if let Some(default) = default {
        picker.default(match default {
            ModLoader::Quilt => 0,
            ModLoader::Fabric => 1,
            ModLoader::Forge => 2,
        });
    }
    match picker.interact()? {
        0 => Ok(ModLoader::Quilt),
        1 => Ok(ModLoader::Fabric),
        2 => Ok(ModLoader::Forge),
        _ => unreachable!(),
    }
}

async fn pick_minecraft_version() -> Result<String> {
    let mut latest_versions: Vec<String> = misc::get_major_mc_versions(10).await?;
    let selected_version = Select::with_theme(&*crate::THEME)
        .with_prompt("Which version of Minecraft do you play?")
        .items(&latest_versions)
        .default(0)
        .interact()?;
    Ok(latest_versions.swap_remove(selected_version))
}

/// Check that there isn't already a profile with the same name
fn check_profile_name(config: &mut config::structs::Config, name: &str) -> Result<()> {
    for profile in &config.profiles {
        if profile.name == name {
            bail!("A profile with name {} already exists", name);
        }
    }
    Ok(())
}
