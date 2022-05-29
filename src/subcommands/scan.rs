use std::{fs, path::PathBuf, sync::Arc};

use anyhow::{bail, Result};
use ferinth::Ferinth;
use furse::Furse;
use libium::config::structs::{Mod, ModIdentifier, ModPlatform, Profile};
use sha1::{Digest, Sha1};

use crate::{CROSS, TICK, YELLOW_TICK};
// TODO? most of this probably belongs in libium
// TODO? asynchronicity?
// TODO error handling
// TODO curseforge (Furse doesn't have api calls related to fingerprints yet)
pub async fn scan(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    profile: &mut Profile,
    preferred_platform: libium::config::structs::ModPlatform,
) -> Result<()> {
    let mut mods_to_add: Vec<Mod> = vec![];
    'file_loop: for mod_file in fs::read_dir(&profile.output_dir)? {
        let mod_path = mod_file?.path();
        if mod_path.extension().is_some() && mod_path.extension().unwrap() == "jar" {
            // search on both platforms to make sure mods don't duplicate across platforms
            let mut found_mods: Vec<Mod> = vec![];
            if let Ok(mod_) = get_modrinth_mod_by_hash(modrinth.clone(), &mod_path).await {
                found_mods.push(mod_)
            }
            if let Ok(mod_) = get_curseforge_mod_by_hash(curseforge.clone(), &mod_path).await {
                found_mods.push(mod_)
            }
            if found_mods.len() == 0 {
                eprintln!(
                    "{} Could not find file {} on any platform",
                    CROSS,
                    mod_path.file_name().and_then(|name| name.to_str()).unwrap()
                );
                continue 'file_loop;
            }
            for mod_ in &found_mods {
                if profile
                    .mods
                    .iter()
                    .any(|iter| iter.name == mod_.name || mod_.identifier == iter.identifier)
                {
                    println!("{} {} is already added", YELLOW_TICK.clone(), mod_.name);
                    continue 'file_loop;
                }
            }
            /*
            if it fails to find a mod on the preferred hosting platfrom it will 
            add the first (and only, as long as no one makes another fancy modding platform) 
            mod in the list
            */
            if !found_mods
                .iter()
                .any(|mod_| match (&preferred_platform, &mod_.identifier) {
                    (ModPlatform::Modrinth, ModIdentifier::ModrinthProject(_)) => {
                        mods_to_add.push(mod_.clone());
                        println!("{} Found mod {} on Modrinth", TICK.clone(), mod_.name);
                        true
                    },
                    (ModPlatform::Curseforge, ModIdentifier::CurseForgeProject(_)) => {
                        mods_to_add.push(mod_.clone());
                        println!("{} Found mod {} on CurseForge", TICK.clone(), mod_.name);
                        true
                    },
                    _ => false,
                })
            {
                mods_to_add.push(found_mods[0].clone());
            }
        }
    }
    profile.mods.extend(mods_to_add);
    Ok(())
}

async fn get_modrinth_mod_by_hash(modrinth: Arc<Ferinth>, mod_path: &PathBuf) -> Result<Mod> {
    let hash = Sha1::default().chain_update(fs::read(mod_path)?).finalize();
    // mod title isn't included in this request
    let version = modrinth
        .get_version_from_file_hash(&format!("{:x}", hash))
        .await?;
    let project = modrinth.get_project(&version.project_id).await?;
    Ok(Mod {
        check_game_version: None,
        check_mod_loader: None,
        identifier: ModIdentifier::ModrinthProject(project.id.clone()),
        name: project.title.clone(),
    })
}

async fn get_curseforge_mod_by_hash(_curseforge: Arc<Furse>, _mod_path: &PathBuf) -> Result<Mod> {
    bail!("TODO")
}
