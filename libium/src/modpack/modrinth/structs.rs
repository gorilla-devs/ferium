use ferinth::structures::{project::ProjectSupportRange, version::Hash, Int};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// The version of the format, stored as a number.
    /// The current value at the time of writing is `1`.
    pub format_version: Int,
    pub game: Game,
    pub version_id: String,
    /// Human readable name of the modpack
    pub name: String,
    /// A short description of this modpack
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// A list of files for the modpack that needs
    pub files: Vec<ModpackFile>,
    /// A list of IDs and version numbers that launchers will use in order to know what to install
    pub dependencies: HashMap<DependencyID, String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyID {
    Minecraft,
    Forge,
    Neoforge,
    FabricLoader,
    QuiltLoader,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModpackFile {
    /// The destination path of this file, relative to the Minecraft instance directory
    pub path: PathBuf,
    pub hashes: Hash,
    /// The specific environment this file exists on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<ModpackFileEnvironment>,
    /// HTTPS URLs where this file may be downloaded
    pub downloads: Vec<Url>,
    /// The size of the file in bytes
    pub file_size: Int,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModpackFileEnvironment {
    client: ProjectSupportRange,
    server: ProjectSupportRange,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Game {
    Minecraft,
}
