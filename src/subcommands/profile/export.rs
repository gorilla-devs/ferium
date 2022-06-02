use anyhow::{bail, Result};
use libium::config::structs::Profile;
use std::{fs::File, path::PathBuf};

pub async fn export(profile: &Profile, output_path: Option<PathBuf>) -> Result<()> {
    let path = if let Some(path) = output_path {
        path
    } else {
        // TODO make a picker for a file in libium rather than a folder
        bail!("File picker doesn't work yet, specify a path manually");
    };

    // Don't export output directory
    let mut profile = profile.clone();
    profile.output_dir = "".into();

    serde_json::ser::to_writer_pretty(File::create(path)?, &profile)?;
    println!("Profile exported");

    Ok(())
}
