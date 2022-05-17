use super::check_output_directory;
use anyhow::Result;
use libium::{config::structs::Modpack, file_picker};
use std::path::PathBuf;

pub async fn configure(modpack: &mut Modpack, output_dir: &Option<PathBuf>) -> Result<()> {
    match output_dir {
        Some(output_dir) => {
            check_output_directory(output_dir).await?;
            modpack.output_dir = output_dir.clone();
        },
        None => {
            if let Some(dir) =
                file_picker::pick_folder(&modpack.output_dir, "Pick an output directory").await
            {
                check_output_directory(&dir).await?;
                modpack.output_dir = dir;
            }
        },
    }
    Ok(())
}
