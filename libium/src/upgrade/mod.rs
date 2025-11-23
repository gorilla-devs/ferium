pub mod check;
pub mod mod_downloadable;
pub mod modpack_downloadable;

use crate::{
    config::{
        filters::ReleaseChannel,
        structs::{ModIdentifier, ModLoader},
    },
    iter_ext::IterExt as _,
    modpack::modrinth::structs::ModpackFile as ModpackModFile,
    version_ext::VersionExt,
};
use ferinth::structures::version::{
    DependencyType as MRDependencyType, Version as MRVersion, VersionType,
};
use furse::structures::file_structs::{
    File as CFFile, FileRelationType as CFFileRelationType, FileReleaseType,
};
use octocrab::models::repos::{Asset as GHAsset, Release as GHRelease};
use reqwest::{Client, Url};
use std::{
    fs::{create_dir_all, rename, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    ReqwestError(#[from] reqwest::Error),
    IOError(#[from] std::io::Error),
}
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Metadata {
    /// The title of the GitHub Release, Modrinth Version, or CurseForge File
    pub title: String,
    /// The body of the GitHub Release, or the changelog of the Modrinth Version
    pub description: String,
    pub filename: String,

    pub channel: ReleaseChannel,

    pub game_versions: Vec<String>,
    pub loaders: Vec<ModLoader>,
}

#[derive(Debug, Clone)]
pub struct DownloadData {
    pub download_url: Url,
    /// The path of the downloaded file relative to the output directory
    ///
    /// The filename by default, but can be configured with subdirectories for modpacks.
    pub output: PathBuf,
    /// The length of the file in bytes
    pub length: usize,
    /// The dependencies this file has
    pub dependencies: Vec<ModIdentifier>,
    /// Other mods this file is incompatible with
    pub conflicts: Vec<ModIdentifier>,
}

#[derive(Debug, thiserror::Error)]
#[error("The developer of this project has denied third party applications from downloading it")]
/// Contains the mod ID and file ID
pub struct DistributionDeniedError(pub i32, pub i32);

pub fn try_from_cf_file(
    file: CFFile,
) -> std::result::Result<(Metadata, DownloadData), DistributionDeniedError> {
    Ok((
        Metadata {
            title: file.display_name,
            description: String::new(), // Changelog requires a separate request
            filename: file.file_name.clone(),
            channel: match file.release_type {
                FileReleaseType::Release => ReleaseChannel::Release,
                FileReleaseType::Beta => ReleaseChannel::Beta,
                FileReleaseType::Alpha => ReleaseChannel::Alpha,
            },
            loaders: file
                .game_versions
                .iter()
                .filter_map(|s| ModLoader::from_str(s).ok())
                .collect_vec(),
            game_versions: file.game_versions,
        },
        DownloadData {
            download_url: file
                .download_url
                .ok_or(DistributionDeniedError(file.mod_id, file.id))?,
            output: file.file_name.as_str().into(),
            length: file.file_length as usize,
            dependencies: file
                .dependencies
                .iter()
                .filter_map(|d| {
                    if d.relation_type == CFFileRelationType::RequiredDependency {
                        Some(ModIdentifier::CurseForgeProject(d.mod_id))
                    } else {
                        None
                    }
                })
                .collect_vec(),
            conflicts: file
                .dependencies
                .iter()
                .filter_map(|d| {
                    if d.relation_type == CFFileRelationType::Incompatible {
                        Some(ModIdentifier::CurseForgeProject(d.mod_id))
                    } else {
                        None
                    }
                })
                .collect_vec(),
        },
    ))
}

pub fn from_mr_version(version: MRVersion) -> (Metadata, DownloadData) {
    (
        Metadata {
            title: version.name.clone(),
            description: version.changelog.as_ref().cloned().unwrap_or_default(),
            filename: version.get_version_file().filename.clone(),
            channel: match version.version_type {
                VersionType::Release => ReleaseChannel::Release,
                VersionType::Beta => ReleaseChannel::Beta,
                VersionType::Alpha => ReleaseChannel::Alpha,
            },
            loaders: version
                .loaders
                .iter()
                .filter_map(|s| ModLoader::from_str(s).ok())
                .collect_vec(),

            game_versions: version.game_versions.clone(),
        },
        DownloadData {
            download_url: version.get_version_file().url.clone(),
            output: version.get_version_file().filename.as_str().into(),
            length: version.get_version_file().size,
            dependencies: version
                .dependencies
                .clone()
                .into_iter()
                .filter_map(|d| {
                    if d.dependency_type == MRDependencyType::Required {
                        match (d.project_id, d.version_id) {
                            (Some(proj_id), Some(ver_id)) => {
                                Some(ModIdentifier::PinnedModrinthProject(proj_id, ver_id))
                            }
                            (Some(proj_id), None) => Some(ModIdentifier::ModrinthProject(proj_id)),
                            _ => {
                                eprintln!("Project ID not available");
                                None
                            }
                        }
                    } else {
                        None
                    }
                })
                .collect_vec(),
            conflicts: version
                .dependencies
                .into_iter()
                .filter_map(|d| {
                    if d.dependency_type == MRDependencyType::Incompatible {
                        match (d.project_id, d.version_id) {
                            (Some(proj_id), Some(ver_id)) => {
                                Some(ModIdentifier::PinnedModrinthProject(proj_id, ver_id))
                            }
                            (Some(proj_id), None) => Some(ModIdentifier::ModrinthProject(proj_id)),
                            _ => {
                                eprintln!("Project ID not available");
                                None
                            }
                        }
                    } else {
                        None
                    }
                })
                .collect_vec(),
        },
    )
}

pub fn from_modpack_file(file: ModpackModFile) -> DownloadData {
    DownloadData {
        download_url: file
            .downloads
            .first()
            .expect("Download URLs not provided")
            .clone(),
        output: file.path,
        length: file.file_size,
        dependencies: Vec::new(),
        conflicts: Vec::new(),
    }
}

pub fn from_gh_releases(
    releases: impl IntoIterator<Item = GHRelease>,
) -> Vec<(Metadata, DownloadData)> {
    releases
        .into_iter()
        .flat_map(|release| {
            release.assets.into_iter().map(move |asset| {
                (
                    Metadata {
                        title: release.name.clone().unwrap_or_default(),
                        description: release.body.clone().unwrap_or_default(),
                        channel: if release.prerelease {
                            ReleaseChannel::Beta
                        } else {
                            ReleaseChannel::Release
                        },
                        game_versions: asset
                            .name
                            .trim_end_matches(".jar")
                            .split(['-', '_', '+'])
                            .map(|s| s.trim_start_matches("mc"))
                            .map(ToOwned::to_owned)
                            .collect_vec(),
                        loaders: asset
                            .name
                            .trim_end_matches(".jar")
                            .split(['-', '_', '+'])
                            .filter_map(|s| ModLoader::from_str(s).ok())
                            .collect_vec(),
                        filename: asset.name.clone(),
                    },
                    DownloadData {
                        download_url: asset.browser_download_url,
                        output: asset.name.into(),
                        length: asset.size as usize,
                        dependencies: Vec::new(),
                        conflicts: Vec::new(),
                    },
                )
            })
        })
        .collect_vec()
}

pub fn from_gh_asset(asset: GHAsset) -> DownloadData {
    DownloadData {
        download_url: asset.browser_download_url,
        output: asset.name.into(),
        length: asset.size as usize,
        dependencies: Vec::new(),
        conflicts: Vec::new(),
    }
}

impl DownloadData {
    /// Consumes `self` and downloads the file to the `output_dir`
    ///
    /// The `update` closure is called with the chunk length whenever a chunk is downloaded and written.
    ///
    /// Returns the total size of the file and the filename.
    pub async fn download(
        self,
        client: Client,
        output_dir: impl AsRef<Path>,
        update: impl Fn(usize) + Send,
    ) -> Result<(usize, String)> {
        let (filename, url, size) = (self.filename(), self.download_url, self.length);
        let out_file_path = output_dir.as_ref().join(&self.output);
        let temp_file_path = out_file_path.with_extension("part");
        if let Some(up_dir) = out_file_path.parent() {
            create_dir_all(up_dir)?;
        }

        let mut temp_file = BufWriter::with_capacity(
            size,
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(&temp_file_path)?,
        );

        let mut response = client.get(url).send().await?;
        response.error_for_status_ref()?;

        while let Some(chunk) = response.chunk().await? {
            temp_file.write_all(&chunk)?;
            update(chunk.len());
        }
        temp_file.flush()?;
        rename(temp_file_path, out_file_path)?;
        Ok((size, filename))
    }

    pub fn filename(&self) -> String {
        self.output
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }
}
