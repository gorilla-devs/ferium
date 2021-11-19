//! Contains deserialisations and wrappers for accesing Mojang's Minecraft version manifest (version 2)

use crate::ferium_error::*;
use reqwest::{get, Response};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct VersionManifestV2 {
    /// IDs of the latest versions of Minecraft
    pub latest: LatestVersions,
    /// All versions of Minecraft
    pub versions: Vec<Version>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LatestVersions {
    /// Latest release of Minecraft's ID
    pub release: String,
    /// Latest snapshot of Minecraft's ID
    pub snapshot: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Version {
    /// Name of version
    pub id: String,
    #[serde(rename = "type")]
    /// Type of version
    pub type_field: VersionType,
    /// URL to version's manifest
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    /// Time when versions was released
    pub release_time: String,
    /// SHA1 hash of the version
    pub sha1: String,
    #[serde(rename = "complianceLevel")]
    /// Whether this version is "historical"
    pub compliance_level: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum VersionType {
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "snapshot")]
    Snapshot,
    #[serde(rename = "old_beta")]
    Beta,
    #[serde(rename = "old_alpha")]
    Alpha,
}

/// Get the version manifest v2 from Mojang
pub async fn get_version_manifest() -> FResult<VersionManifestV2> {
    Ok(
        request("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")
            .await?
            .json()
            .await?,
    )
}

/// Send a request to `url` with `client` and return response
async fn request(url: &str) -> FResult<Response> {
    match get(url).await?.error_for_status() {
        Ok(response) => Ok(response),
        Err(err) => Err(err.into())
    }
}
