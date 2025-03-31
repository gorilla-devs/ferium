use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    /// Information about how to setup Minecraft
    pub minecraft: Minecraft,
    /// The type of this manifest ??
    pub manifest_type: ManifestType,
    pub manifest_version: i32,
    pub name: String,
    pub version: String,
    pub author: String,
    /// The files this modpack needs
    pub files: Vec<ModpackFile>,
    /// A directory of overrides to install
    pub overrides: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Minecraft {
    pub version: String,
    /// A list of mod loaders that can be used
    pub mod_loaders: Vec<ModpackModLoader>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ManifestType {
    MinecraftModpack,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ModpackFile {
    #[serde(rename = "projectID")]
    pub project_id: i32,
    #[serde(rename = "fileID")]
    pub file_id: i32,
    pub required: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModpackModLoader {
    /// The name/ID of the mod loader
    pub id: String,
    /// Whether this is the recommended mod loader
    pub primary: bool,
}
