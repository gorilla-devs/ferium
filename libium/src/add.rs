use crate::{
    config::{
        filters::{Filter, ReleaseChannel},
        structs::{ModIdentifier, ModLoader, Profile},
    },
    iter_ext::IterExt as _,
    upgrade::{check, Metadata},
    CURSEFORGE_API, GITHUB_API, MODRINTH_API,
};
use serde::Deserialize;
use std::{collections::HashMap, str::FromStr};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(
        "The developer of this project has denied third party applications from downloading it"
    )]
    /// The user can manually download the mod and place it in the `user` folder of the output directory to mitigate this.
    /// However, they will have to manually update the mod.
    DistributionDenied,
    #[error("The project has already been added")]
    AlreadyAdded,
    #[error("The project is not compatible because {0}")]
    Incompatible(#[from] check::Error),
    #[error("The project does not exist")]
    DoesNotExist,
    #[error("The project is not a mod")]
    NotAMod,
    #[error("The specified version pin does not exist for this mod")]
    IncorrectVersionPin,
    #[error("The identifier provided is not in the correct format")]
    InvalidIdentifier,
    #[error("GitHub: {0}")]
    GitHubError(String),
    #[error("GitHub: {0:#?}")]
    OctocrabError(#[from] octocrab::Error),
    #[error("Modrinth: {0}")]
    ModrinthError(#[from] ferinth::Error),
    #[error("CurseForge: {0}")]
    CurseForgeError(#[from] furse::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize, Debug)]
struct GraphQlResponse {
    data: HashMap<String, Option<ResponseData>>,
    #[serde(default)]
    errors: Vec<GraphQLError>,
}

#[derive(Deserialize, Debug)]
struct GraphQLError {
    #[serde(rename = "type")]
    type_: String,
    path: Vec<String>,
    message: String,
}

#[derive(Deserialize, Debug)]
struct ResponseData {
    owner: OwnerData,
    name: String,
    releases: ReleaseConnection,
}
#[derive(Deserialize, Debug)]
struct OwnerData {
    login: String,
}
#[derive(Deserialize, Debug)]
struct ReleaseConnection {
    nodes: Vec<Release>,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Release {
    name: String,
    description: String,
    is_prerelease: bool,
    release_assets: ReleaseAssetConnection,
}
#[derive(Deserialize, Debug)]
struct ReleaseAssetConnection {
    nodes: Vec<ReleaseAsset>,
}
#[derive(Deserialize, Debug)]
struct ReleaseAsset {
    id: i64,
    name: String,
}

pub fn parse_id(id: String) -> Result<ModIdentifier> {
    let split = id.split('-').collect_vec();
    let (id, pin) = match split.as_slice() {
        [id, pin] => (id, Some(pin)),
        [id] => (id, None),
        _ => return Err(Error::InvalidIdentifier),
    };

    if let Ok(id) = id.parse() {
        Ok(ModIdentifier::CurseForgeProject(
            id,
            if let Some(pin) = pin {
                Some(pin.parse().map_err(|_| Error::InvalidIdentifier)?)
            } else {
                None
            },
        ))
    } else {
        let split = id.split('/').collect_vec();
        if let [owner, repo] = split.as_slice() {
            Ok(ModIdentifier::GitHubRepository(
                (owner.to_string(), repo.to_string()),
                if let Some(pin) = pin {
                    Some(pin.parse().map_err(|_| Error::InvalidIdentifier)?)
                } else {
                    None
                },
            ))
        } else {
            Ok(ModIdentifier::ModrinthProject(
                id.to_string(),
                pin.map(ToString::to_string),
            ))
        }
    }
}

/// Adds mods from `identifiers`, and returns successful mods with their names, and unsuccessful mods with an error.
/// Currently does not batch requests when adding multiple pinned mods.
///
/// Classifies the `identifiers` into the appropriate platforms, sends batch requests to get the necessary information,
/// checks details about the projects, and adds them to `profile` if suitable.
/// Performs checks on the mods to see whether they're compatible with the profile if `perform_checks` is true
pub async fn add(
    profile: &mut Profile,
    identifiers: Vec<ModIdentifier>,
    perform_checks: bool,
    override_profile: bool,
    filters: Vec<Filter>,
) -> Result<(Vec<(String, ModIdentifier)>, Vec<(String, Error)>)> {
    let mut mr_project_ids = Vec::new();
    let mut mr_version_ids = Vec::new();

    let mut cf_project_ids = Vec::new();
    let mut cf_file_ids = Vec::new();

    let mut gh_repo_ids = Vec::new();
    let mut gh_asset_ids = Vec::new();

    let mut errors = Vec::new();

    for id in identifiers {
        match id {
            ModIdentifier::CurseForgeProject(p, v) => {
                cf_project_ids.push(p);
                cf_file_ids.push(v);
            }
            ModIdentifier::ModrinthProject(p, v) => {
                mr_project_ids.push(p);
                mr_version_ids.push(v);
            }
            ModIdentifier::GitHubRepository(p, v) => {
                gh_repo_ids.push(p);
                gh_asset_ids.push(v);
            }
        }
    }

    let cf_projects = if !cf_project_ids.is_empty() {
        CURSEFORGE_API.get_mods(cf_project_ids.clone()).await?
    } else {
        Vec::new()
    };

    let cf_files = if cf_file_ids.iter().any(Option::is_some) {
        CURSEFORGE_API
            .get_files(cf_file_ids.iter().copied().flatten().collect_vec())
            .await?
    } else {
        Vec::new()
    };

    let mr_projects = if !mr_project_ids.is_empty() {
        MODRINTH_API
            .project_get_multiple(&mr_project_ids.iter().map(AsRef::as_ref).collect_vec())
            .await?
    } else {
        Vec::new()
    };

    let gh_repos = if !gh_repo_ids.is_empty() {
        // Construct GraphQl query using raw strings
        let mut graphql_query = "{".to_string();
        for (i, (owner, name)) in gh_repo_ids.iter().enumerate() {
            graphql_query.push_str(&format!(
                r#"_{i}: repository(owner: "{owner}", name: "{name}") {{
                    owner {{
                        login
                    }}
                    name
                    releases(first: 100) {{
                        nodes {{
                            name
                            description
                            isPrerelease
                            releaseAssets(first: 10) {{
                                nodes {{
                                    id
                                    name
                                }}
                            }}
                        }}
                    }}
                }}"#
            ));
        }
        graphql_query.push('}');

        // Send the query
        let response: GraphQlResponse = GITHUB_API
            .graphql(&serde_json::json!({
                "query": graphql_query
            }))
            .await?;

        errors.extend(response.errors.into_iter().map(|err| {
            (
                {
                    let (owner, repo) = &gh_repo_ids[err.path[0]
                        .strip_prefix('_')
                        .and_then(|s| s.parse::<usize>().ok())
                        .expect("Unexpected response data")];
                    format!("{owner}/{repo}")
                },
                if err.type_ == "NOT_FOUND" {
                    Error::DoesNotExist
                } else {
                    Error::GitHubError(err.message)
                },
            )
        }));

        response
            .data
            .into_values()
            .flatten()
            .map(|d| ((d.owner.login, d.name), d.releases.nodes))
            .collect_vec()
    } else {
        Vec::new()
    };

    let mut success_names = Vec::new();

    for (project, pin) in cf_projects.into_iter().zip(cf_file_ids) {
        if let Some(i) = cf_project_ids.iter().position(|&id| id == project.id) {
            cf_project_ids.swap_remove(i);
        }

        let res = 'cf_check: {
            let identifier = ModIdentifier::CurseForgeProject(project.id, pin);

            if profile.mods.iter().any(|mod_| {
                mod_.name.eq_ignore_ascii_case(&project.name)
                    || mod_.identifier.is_same_as(&identifier)
            }) {
                break 'cf_check Err(Error::AlreadyAdded);
            }

            // Check if it can be downloaded by third-parties
            if Some(false) == project.allow_mod_distribution {
                break 'cf_check Err(Error::DistributionDenied);
            }

            // Check if the project is a Minecraft mod
            if !project.links.website_url.as_str().contains("mc-mods") {
                break 'cf_check Err(Error::NotAMod);
            }

            // Check if the mod is compatible
            if let Some(pin) = pin {
                if let Some(file) = cf_files.iter().flatten().find(|file| file.id == pin) {
                    if file.mod_id != project.id {
                        break 'cf_check Err(Error::IncorrectVersionPin);
                    }
                } else {
                    break 'cf_check Err(Error::IncorrectVersionPin);
                }
            } else if perform_checks {
                // A very rough check that uses the latest file indexes (which to my knowledge
                // always contain all possible mod loader and game version combinations)
                // to only check that the
                check::select_latest(
                    [Metadata {
                        filename: String::new(),
                        title: String::new(),
                        description: String::new(),
                        game_versions: project
                            .latest_files_indexes
                            .iter()
                            .map(|i| i.game_version.clone())
                            .collect_vec(),
                        loaders: project
                            .latest_files_indexes
                            .iter()
                            .filter_map(|i| {
                                i.mod_loader
                                    .as_ref()
                                    .and_then(|l| ModLoader::from_str(&format!("{l:?}")).ok())
                            })
                            .collect_vec(),
                        channel: ReleaseChannel::Release,
                    }]
                    .iter(),
                    if override_profile {
                        profile.filters.clone()
                    } else {
                        [profile.filters.clone(), filters.clone().clone()].concat()
                    }
                    .iter()
                    .filter(|f| {
                        matches!(
                            f,
                            Filter::GameVersionStrict(_)
                                | Filter::GameVersionMinor(_)
                                | Filter::ModLoaderAny(_)
                                | Filter::ModLoaderPrefer(_)
                        )
                    })
                    .cloned()
                    .collect_vec(),
                )
                .await?;
            }

            profile.push_mod(
                project.name.trim().to_string(),
                identifier.clone(),
                project.slug.clone(),
                override_profile,
                filters.clone(),
            );

            Ok(identifier)
        };
        match res {
            Ok(id) => success_names.push((project.name, id)),
            Err(err) => errors.push((format!("{} ({})", project.name, project.id), err)),
        }
    }
    errors.extend(
        cf_project_ids
            .into_iter()
            .map(|id| (id.to_string(), Error::DoesNotExist)),
    );

    for (project, pin) in mr_projects.into_iter().zip(mr_version_ids) {
        if let Some(i) = mr_project_ids
            .iter()
            .position(|id| id == &project.id || project.slug.eq_ignore_ascii_case(id))
        {
            mr_project_ids.swap_remove(i);
        }

        let res = 'mr_check: {
            let identifier = ModIdentifier::ModrinthProject(project.id.clone(), pin.clone());

            if profile.mods.iter().any(|mod_| {
                mod_.name.eq_ignore_ascii_case(&project.title)
                    || mod_.identifier.is_same_as(&identifier)
            }) {
                break 'mr_check Err(Error::AlreadyAdded);
            }

            if project.project_type != ferinth::structures::project::ProjectType::Mod {
                break 'mr_check Err(Error::NotAMod);
            }

            if let Some(pin) = &pin {
                if !project.versions.contains(pin) {
                    break 'mr_check Err(Error::IncorrectVersionPin);
                }
            } else if perform_checks {
                check::select_latest(
                    [Metadata {
                        filename: String::new(),
                        title: String::new(),
                        description: String::new(),
                        game_versions: project.game_versions.clone(),
                        loaders: project
                            .loaders
                            .iter()
                            .filter_map(|s| ModLoader::from_str(s).ok())
                            .collect_vec(),
                        channel: ReleaseChannel::Release,
                    }]
                    .iter(),
                    if override_profile {
                        profile.filters.clone()
                    } else {
                        [profile.filters.clone(), filters.clone().clone()].concat()
                    }
                    .iter()
                    .filter(|f| {
                        matches!(
                            f,
                            Filter::GameVersionStrict(_)
                                | Filter::GameVersionMinor(_)
                                | Filter::ModLoaderAny(_)
                                | Filter::ModLoaderPrefer(_)
                        )
                    })
                    .cloned()
                    .collect_vec(),
                )
                .await?;
            }

            profile.push_mod(
                project.title.trim().to_owned(),
                identifier.clone(),
                project.slug.to_owned(),
                override_profile,
                filters.clone(),
            );

            Ok(identifier)
        };
        match res {
            Ok(id) => success_names.push((project.title, id)),
            Err(err) => errors.push((format!("{} ({})", project.title, project.id), err)),
        }
    }
    errors.extend(
        mr_project_ids
            .into_iter()
            .map(|id| (id, Error::DoesNotExist)),
    );

    for (((owner, repo), releases), pin) in gh_repos.into_iter().zip(gh_asset_ids) {
        let res = 'gh_check: {
            let identifier = ModIdentifier::GitHubRepository((owner.clone(), repo.clone()), pin);

            if profile.mods.iter().any(|mod_| {
                mod_.name.eq_ignore_ascii_case(repo.as_ref())
                    || matches!(
                        &mod_.identifier,
                        ModIdentifier::GitHubRepository((o, r), _) if o == &owner && r == &repo,
                    )
            }) {
                break 'gh_check Err(Error::AlreadyAdded);
            }
            if let Some(pin) = pin {
                if !releases
                    .into_iter()
                    .flat_map(|release| release.release_assets.nodes)
                    .any(|asset| asset.id == pin)
                {
                    break 'gh_check Err(Error::IncorrectVersionPin);
                }
            } else if perform_checks {
                // Check if the repo is compatible
                check::select_latest(
                    releases
                        .into_iter()
                        .flat_map(|release| {
                            release
                                .release_assets
                                .nodes
                                .into_iter()
                                .map(move |asset| Metadata {
                                    title: release.name.clone(),
                                    description: release.description.clone(),
                                    channel: if release.is_prerelease {
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
                                    filename: asset.name,
                                })
                        })
                        .collect_vec()
                        .iter(),
                    if override_profile {
                        profile.filters.clone()
                    } else {
                        [profile.filters.clone(), filters.clone()].concat()
                    },
                )
                .await?;
            }

            profile.push_mod(
                repo.clone(),
                identifier.clone(),
                repo.clone(),
                override_profile,
                filters.clone(),
            );

            Ok(identifier)
        };
        match res {
            Ok(id) => success_names.push((format!("{owner}/{repo}"), id)),
            Err(err) => errors.push((format!("{owner}/{repo}"), err)),
        }
    }

    Ok((success_names, errors))
}
