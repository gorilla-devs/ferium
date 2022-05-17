use crate::TICK;
use anyhow::Result;
use ferinth::Ferinth;
use furse::Furse;
use libium::{config::structs::Config, modpack::add};
use std::{path::PathBuf, sync::Arc};

pub async fn curseforge(
    curseforge: Arc<Furse>,
    config: &mut Config,
    project_id: i32,
    output_dir: PathBuf,
) -> Result<()> {
    eprint!("Adding modpack... ");
    let project = add::curseforge(curseforge, config, project_id, output_dir).await?;
    println!("{} ({})", *TICK, project.name);
    Ok(())
}

pub async fn modrinth(
    modrinth: Arc<Ferinth>,
    config: &mut Config,
    project_id: &str,
    output_dir: PathBuf,
) -> Result<()> {
    eprint!("Adding modpack... ");
    let project = add::modrinth(modrinth, config, project_id, output_dir).await?;
    println!("{} ({})", *TICK, project.title);
    Ok(())
}
