pub mod structs;

use std::{
    fs::{canonicalize, read_dir, File},
    io::{copy, Write},
    path::{Path, PathBuf},
};

use zip::{write::SimpleFileOptions, ZipWriter};
use zip_extensions::ZipWriterExtensions;

/// Create a Modrinth modpack at `output` using the provided `metadata` and optional `overrides`
pub fn create(
    output: &Path,
    metadata: &str,
    overrides: Option<&Path>,
    additional_mods: Option<&Path>,
) -> zip::result::ZipResult<()> {
    let mut writer = ZipWriter::new(File::create(output)?);
    let options = SimpleFileOptions::default();

    // Add metadata to the zip file
    writer.start_file("modrinth.index.json", options)?;
    writer.write_all(metadata.as_bytes())?;

    // Add additional (non-Modrinth) mods to the zip file
    if let Some(path) = additional_mods {
        for entry in read_dir(path)?
            .flatten()
            .filter(|entry| entry.file_type().map(|e| e.is_file()).unwrap_or(false))
        {
            let entry = canonicalize(entry.path())?;
            writer.start_file(
                PathBuf::from("overrides")
                    .join("mods")
                    .with_file_name(entry.file_name().unwrap())
                    .to_string_lossy(),
                options,
            )?;
            copy(&mut File::open(entry)?, &mut writer)?;
        }
    }

    // Add the overrides to the zip file
    if let Some(overrides) = overrides {
        writer.create_from_directory(&overrides.to_owned())?;
    }

    Ok(())
}
