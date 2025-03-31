pub mod filters;
pub mod structs;

use std::{
    fs::{create_dir_all, File},
    io::{BufReader, Result},
    path::{Path, PathBuf},
    sync::LazyLock,
};

pub static DEFAULT_CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    crate::HOME
        .join(".config")
        .join("ferium")
        .join("config.json")
});

/// Open the config file at `path` and deserialise it into a config struct
pub fn read_config(path: impl AsRef<Path>) -> Result<structs::Config> {
    if !path.as_ref().exists() {
        create_dir_all(path.as_ref().parent().expect("Invalid config directory"))?;
        write_config(&path, &structs::Config::default())?;
    }

    let config_file = BufReader::new(File::open(&path)?);
    let mut config: structs::Config = serde_json::from_reader(config_file)?;

    config
        .profiles
        .iter_mut()
        .for_each(structs::Profile::backwards_compat);

    Ok(config)
}

/// Serialise `config` and write it to the config file at `path`
pub fn write_config(path: impl AsRef<Path>, config: &structs::Config) -> Result<()> {
    let config_file = File::create(path)?;
    serde_json::to_writer_pretty(config_file, config)?;
    Ok(())
}
