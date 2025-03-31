use super::{from_mr_version, try_from_cf_file, DistributionDeniedError};
use crate::{config::structs::ModpackIdentifier, CURSEFORGE_API, HOME, MODRINTH_API};
use reqwest::Client;
use std::{fs::create_dir_all, path::PathBuf};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    /// The user can manually download the modpack zip file and place it in `~/.config/ferium/.cache/` to mitigate this.
    /// However, they will have to manually update the modpack file.
    DistributionDenied(#[from] DistributionDeniedError),
    ModrinthError(#[from] ferinth::Error),
    CurseForgeError(#[from] furse::Error),
    ReqwestError(#[from] reqwest::Error),
    DownloadError(#[from] super::Error),
    IOError(#[from] std::io::Error),
}
type Result<T> = std::result::Result<T, Error>;

impl ModpackIdentifier {
    pub async fn download_file(
        &self,
        total: impl FnOnce(usize) + Send,
        update: impl Fn(usize) + Send,
    ) -> Result<PathBuf> {
        let (_, download_data) = match self {
            ModpackIdentifier::CurseForgeModpack(id) => {
                try_from_cf_file(CURSEFORGE_API.get_mod_files(*id).await?.swap_remove(0))?
            }
            ModpackIdentifier::ModrinthModpack(id) => {
                from_mr_version(MODRINTH_API.list_versions(id).await?.swap_remove(0))
            }
        };

        let cache_dir = HOME.join(".config").join("ferium").join(".cache");
        let modpack_path = cache_dir.join(&download_data.output);
        if !modpack_path.exists() {
            create_dir_all(&cache_dir)?;
            total(download_data.length);
            download_data
                .download(Client::new(), &cache_dir, update)
                .await?;
        }

        Ok(modpack_path)
    }
}
