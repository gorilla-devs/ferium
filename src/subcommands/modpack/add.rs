use crate::TICK;
use anyhow::{anyhow, Result};
use furse::Furse;
use libium::{config::structs::Config, file_picker, misc::get_minecraft_dir, modpack::add};
use std::sync::Arc;

pub async fn curseforge(
    curseforge: Arc<Furse>,
    config: &mut Config,
    project_id: i32,
) -> Result<()> {
    println!("Where should the modpack be installed to?");
    let output_dir = file_picker::pick_folder(&get_minecraft_dir(), "Pick an output directory")
        .await
        .ok_or_else(|| anyhow!("Please pick an output directory"))?;
    eprint!("Adding modpack... ");
    let (project, _) = add::curseforge(curseforge, config, project_id, output_dir).await?;
    println!("{} ({})", *TICK, project.name);
    Ok(())
}
