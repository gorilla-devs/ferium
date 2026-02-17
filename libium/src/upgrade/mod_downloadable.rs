use super::{
    from_gh_asset, from_gh_releases, from_mr_version, try_from_cf_file, DistributionDeniedError,
    DownloadData,
};
use crate::{
    config::{
        filters::Filter,
        structs::{Mod, ModIdentifier},
    },
    iter_ext::IterExt as _,
    CURSEFORGE_API, GITHUB_API, MODRINTH_API,
};
use std::cmp::Reverse;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    DistributionDenied(#[from] DistributionDeniedError),
    CheckError(#[from] super::check::Error),
    #[error("The pin provided is an invalid identifier")]
    InvalidPinID,
    #[error("Modrinth: {0}")]
    ModrinthError(#[from] ferinth::Error),
    #[error("CurseForge: {0}")]
    CurseForgeError(#[from] furse::Error),
    #[error("GitHub: {0:#?}")]
    GitHubError(#[from] octocrab::Error),
}
type Result<T> = std::result::Result<T, Error>;

impl Mod {
    pub async fn fetch_download_file(&self, profile_filters: Vec<Filter>) -> Result<DownloadData> {
        match &self.identifier {
            ModIdentifier::CurseForgeProject(mod_id, Some(pin)) => {
                Ok(try_from_cf_file(CURSEFORGE_API.get_mod_file(*mod_id, *pin).await?)?.1)
            }
            ModIdentifier::ModrinthProject(_, Some(pin)) => {
                Ok(from_mr_version(MODRINTH_API.version_get(pin).await?).1)
            }
            ModIdentifier::GitHubRepository((owner, repo), Some(pin)) => Ok(from_gh_asset(
                GITHUB_API
                    .repos(owner, repo)
                    .releases()
                    .list()
                    .send()
                    .await?
                    .items
                    .into_iter()
                    .flat_map(|release| release.assets)
                    .find(|asset| &asset.node_id == pin)
                    .ok_or(Error::InvalidPinID)?,
            )),
            id => {
                let download_files = match &id {
                    ModIdentifier::CurseForgeProject(id, None) => {
                        let mut files = CURSEFORGE_API.get_mod_files(*id).await?;
                        files.sort_unstable_by_key(|f| Reverse(f.file_date));
                        files
                            .into_iter()
                            .map(|f| try_from_cf_file(f).map_err(Into::into))
                            .collect::<Result<Vec<_>>>()?
                    }
                    ModIdentifier::ModrinthProject(id, None) => MODRINTH_API
                        .version_list(id)
                        .await?
                        .into_iter()
                        .map(from_mr_version)
                        .collect_vec(),
                    ModIdentifier::GitHubRepository((owner, repo), None) => GITHUB_API
                        .repos(owner, repo)
                        .releases()
                        .list()
                        .send()
                        .await
                        .map(|r| from_gh_releases(r.items))?,
                    _ => unreachable!(),
                };

                let index = super::check::select_latest(
                    download_files.iter().map(|(m, _)| m),
                    if self.override_filters {
                        self.filters.clone()
                    } else {
                        [profile_filters.clone(), self.filters.clone().clone()].concat()
                    },
                )
                .await?;
                Ok(download_files.into_iter().nth(index).unwrap().1)
            }
        }
    }
}
