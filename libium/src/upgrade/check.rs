use super::Metadata;
use crate::{
    config::filters::{Filter, ReleaseChannel},
    iter_ext::{IterExt, IterExtPositions},
    MODRINTH_API,
};
use ferinth::structures::tag::GameVersionType;
use regex::Regex;
use std::{collections::HashSet, sync::OnceLock};

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum Error {
    VersionGrouping(#[from] ferinth::Error),
    FilenameRegex(#[from] regex::Error),
    #[error("The following filter(s) were empty: {}", _0.iter().display(", "))]
    FilterEmpty(Vec<String>),
    #[error("Failed to find a compatible combination")]
    IntersectFailure,
}
pub type Result<T> = std::result::Result<T, Error>;

static VERSION_GROUPS: OnceLock<Vec<Vec<String>>> = OnceLock::new();

/// Gets groups of versions that are considered minor updates in terms of mod compatibility
///
/// This is determined by Modrinth's `major` parameter for game versions.
pub async fn get_version_groups() -> Result<&'static Vec<Vec<String>>> {
    if let Some(v) = VERSION_GROUPS.get() {
        Ok(v)
    } else {
        let versions = MODRINTH_API.list_game_versions().await?;
        let mut v = vec![vec![]];
        for version in versions {
            if version.version_type == GameVersionType::Release {
                // Push the version to the latest group
                v.last_mut().unwrap().push(version.version);
                // Create a new group if a new major versions is present
                if version.major {
                    v.push(vec![]);
                }
            }
        }
        let _ = VERSION_GROUPS.set(v);

        Ok(VERSION_GROUPS.get().unwrap())
    }
}

impl Filter {
    /// Returns the indices of `download_files` that have successfully filtered through `self`
    ///
    /// This function fails if getting version groups fails, or the regex files to parse.
    pub async fn filter(
        &self,
        download_files: impl Iterator<Item = (usize, &Metadata)> + Clone,
    ) -> Result<HashSet<usize>> {
        Ok(match self {
            Filter::ModLoaderPrefer(loaders) => loaders
                .iter()
                .map(move |l| {
                    download_files
                        .clone()
                        .positions(|f| f.loaders.contains(l))
                        .collect_hashset()
                })
                .find(|v| !v.is_empty())
                .unwrap_or_default(),

            Filter::ModLoaderAny(loaders) => download_files
                .positions(|f| loaders.iter().any(|l| f.loaders.contains(l)))
                .collect_hashset(),

            Filter::GameVersionStrict(versions) => download_files
                .positions(|f| versions.iter().any(|vc| f.game_versions.contains(vc)))
                .collect_hashset(),

            Filter::GameVersionMinor(versions) => {
                let mut final_versions = vec![];
                for group in get_version_groups().await? {
                    if group.iter().any(|v| versions.contains(v)) {
                        final_versions.extend(group.clone());
                    }
                }

                download_files
                    .positions(|f| final_versions.iter().any(|vc| f.game_versions.contains(vc)))
                    .collect_hashset()
            }

            Filter::ReleaseChannel(channel) => download_files
                .positions(|f| match channel {
                    ReleaseChannel::Alpha => true,
                    ReleaseChannel::Beta => {
                        f.channel == ReleaseChannel::Beta || f.channel == ReleaseChannel::Release
                    }
                    ReleaseChannel::Release => f.channel == ReleaseChannel::Release,
                })
                .collect_hashset(),

            Filter::Filename(regex) => {
                let regex = Regex::new(regex)?;
                download_files
                    .positions(|f| regex.is_match(&f.filename))
                    .collect_hashset()
            }

            Filter::Title(regex) => {
                let regex = Regex::new(regex)?;
                download_files
                    .positions(|f| regex.is_match(&f.title))
                    .collect_hashset()
            }

            Filter::Description(regex) => {
                let regex = Regex::new(regex)?;
                download_files
                    .positions(|f| regex.is_match(&f.description))
                    .collect_hashset()
            }
        })
    }
}

/// Assumes that the provided `download_files` are sorted in the order of preference (e.g. chronological)
pub async fn select_latest(
    download_files: impl Iterator<Item = &Metadata> + Clone,
    filters: Vec<Filter>,
) -> Result<usize> {
    let mut filter_results = vec![];
    let mut run_last = vec![];

    for filter in &filters {
        if let Filter::ModLoaderPrefer(_) = filter {
            // ModLoaderPrefer has to be run last
            run_last.push((
                filter,
                filter.filter(download_files.clone().enumerate()).await?,
            ));
        } else {
            filter_results.push((
                filter,
                filter.filter(download_files.clone().enumerate()).await?,
            ));
        }
    }

    let empty_filtrations = filter_results
        .iter()
        .chain(run_last.iter())
        .filter_map(|(filter, indices)| {
            if indices.is_empty() {
                Some(filter.to_string())
            } else {
                None
            }
        })
        .collect_vec();
    if !empty_filtrations.is_empty() {
        return Err(Error::FilterEmpty(empty_filtrations));
    }

    // Get the indices of the filtrations
    let mut filter_results = filter_results.into_iter().map(|(_, set)| set);

    // Intersect all the index_sets by folding the HashSet::intersection method
    // Ref: https://www.reddit.com/r/rust/comments/5v35l6/intersection_of_more_than_two_sets
    // Here we're getting the non-ModLoaderPrefer indices first
    let final_indices = filter_results
        .next()
        .map(|set_1| {
            filter_results.fold(set_1, |set_a, set_b| {
                set_a.intersection(&set_b).copied().collect_hashset()
            })
        })
        .unwrap_or_default();

    let download_files = download_files.into_iter().enumerate().filter_map(|(i, f)| {
        if final_indices.contains(&i) {
            Some((i, f))
        } else {
            None
        }
    });

    let mut filter_results = vec![];
    for (filter, _) in run_last {
        filter_results.push(filter.filter(download_files.clone()).await?)
    }
    let mut filter_results = filter_results.into_iter();

    let final_index = filter_results
        .next()
        .and_then(|set_1| {
            filter_results
                .fold(set_1, |set_a, set_b| {
                    set_a.intersection(&set_b).copied().collect_hashset()
                })
                .into_iter()
                .min()
        })
        .ok_or(Error::IntersectFailure)?;

    Ok(final_index)
}
