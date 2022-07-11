use std::{ffi::OsStr, fs, sync::Arc};

use anyhow::{bail, Result};
use colored::Colorize;
use ferinth::Ferinth;
use furse::Furse;
use libium::{
    add,
    config::structs::{Mod, ModIdentifier, ModPlatform, Profile},
    scan,
};

use crate::{CROSS, TICK, YELLOW_TICK};
pub async fn scan(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    profile: &mut Profile,
    preferred_platform: libium::config::structs::ModPlatform,
) -> Result<()> {
    let mods = scan::scan(
        modrinth.clone(),
        curseforge.clone(),
        fs::read_dir(&profile.output_dir)?
            .filter_map(|path| {
                if let Ok(entry) = path {
                    let file_path = entry.path();
                    if matches!(file_path.extension().and_then(OsStr::to_str), Some("jar")) {
                        Some(file_path)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect(),
    )
    .await?;
    let mut mods_to_add: Vec<(std::path::PathBuf, ModIdentifier)> = vec![];
    for (path, mod_) in mods {
        if !matches!(mod_, (None, None)) {
            let mod_to_add = match (&mod_, &preferred_platform) {
                ((Some(modrinth_mod), _), ModPlatform::Modrinth) => {
                    ModIdentifier::ModrinthProject(modrinth_mod.project_id.clone())
                },
                ((_, Some(curseforge_mod)), ModPlatform::Curseforge) => {
                    ModIdentifier::CurseForgeProject(curseforge_mod.mod_id)
                },
                _ => match &mod_ {
                    (Some(modrinth_mod), _) => {
                        ModIdentifier::ModrinthProject(modrinth_mod.project_id.clone())
                    },
                    (_, Some(curseforge_mod)) => {
                        ModIdentifier::CurseForgeProject(curseforge_mod.mod_id)
                    },
                    _ => unreachable!(),
                },
            };
            mods_to_add.push((path, mod_to_add));
        } else {
            eprintln!(
                "{}",
                format!(
                    "{} Could not find {} on any platform",
                    CROSS,
                    path.display()
                )
                .red()
            );
        }
    }
    for (path, mod_) in mods_to_add {
        match add_mod(modrinth.clone(), curseforge.clone(), mod_, profile).await {
            Ok(mod_) => {
                println!(
                    "{} found {} on {}",
                    TICK.clone(),
                    &mod_.name,
                    match &mod_.identifier {
                        ModIdentifier::CurseForgeProject(_) => "CurseForge",
                        ModIdentifier::ModrinthProject(_) => "Modrinth",
                        _ => unreachable!(),
                    }
                );
                profile.mods.push(mod_);
            },
            Err(add::Error::AlreadyAdded) => {
                println!(
                    "{} {} is already added",
                    YELLOW_TICK.clone(),
                    path.display()
                )
            },
            Err(err) => bail!(err),
        }
    }
    Ok(())
}

async fn add_mod(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    mod_: ModIdentifier,
    profile: &Profile,
) -> Result<Mod, add::Error> {
    match &mod_ {
        ModIdentifier::ModrinthProject(id) => {
            let (project, _version) = add::modrinth(modrinth, id, profile, None, None).await?;
            Ok(Mod {
                check_game_version: None,
                check_mod_loader:   None,
                identifier:         mod_,
                name:               project.title,
            })
        },
        ModIdentifier::CurseForgeProject(id) => {
            let (project, _file) = add::curseforge(curseforge, *id, profile, None, None).await?;
            Ok(Mod {
                check_game_version: None,
                check_mod_loader:   None,
                identifier:         mod_,
                name:               project.name,
            })
        },
        _ => unreachable!(),
    }
}
