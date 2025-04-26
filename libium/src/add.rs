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
    #[error("The project is not compatible because {_0}")]
    Incompatible(#[from] check::Error),
    #[error("The project does not exist")]
    DoesNotExist,
    #[error("The project is not a mod")]
    NotAMod,
    #[error("GitHub: {0}")]
    GitHubError(String),
    #[error("GitHub: {0:#?}")]
    OctocrabError(#[from] octocrab::Error),
    #[error("Modrinth: {0}")]
    ModrinthError(#[from] ferinth::Error),
    #[error("CurseForge: {0}")]
    CurseForgeError(#[from] furse::Error),
}
type Result<T> = std::result::Result<T, Error>;

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
    name: String,
}

pub fn parse_id(id: String) -> ModIdentifier {
    if let Ok(id) = id.parse() {
        ModIdentifier::CurseForgeProject(id)
    } else {
        let split = id.split('/').collect_vec();
        if split.len() == 2 {
            ModIdentifier::GitHubRepository(split[0].to_owned(), split[1].to_owned())
        } else {
            ModIdentifier::ModrinthProject(id)
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
) -> Result<(Vec<String>, Vec<(String, Error)>)> {
    let mut mr_ids = Vec::new();
    let mut cf_ids = Vec::new();
    let mut gh_ids = Vec::new();
    let mut errors = Vec::new();

    for id in identifiers {
        match id {
            ModIdentifier::CurseForgeProject(id) => cf_ids.push(id),
            ModIdentifier::ModrinthProject(id) => mr_ids.push(id),
            ModIdentifier::GitHubRepository(o, r) => gh_ids.push((o, r)),

            ModIdentifier::PinnedCurseForgeProject(mod_id, file_id) => {
                let project = CURSEFORGE_API.get_mod(mod_id).await?;
                let file = CURSEFORGE_API.get_mod_file(mod_id, file_id).await?;
            }
            ModIdentifier::PinnedModrinthProject(project_id, version_id) => todo!(),
            ModIdentifier::PinnedGitHubRepository((owner, repo), asset_id) => todo!(),
        }
    }

    let cf_projects = if !cf_ids.is_empty() {
        cf_ids.sort_unstable();
        cf_ids.dedup();
        CURSEFORGE_API.get_mods(cf_ids.clone()).await?
    } else {
        Vec::new()
    };

    let mr_projects = if !mr_ids.is_empty() {
        mr_ids.sort_unstable();
        mr_ids.dedup();
        MODRINTH_API
            .project_get_multiple(&mr_ids.iter().map(AsRef::as_ref).collect_vec())
            .await?
    } else {
        Vec::new()
    };

    let gh_repos =
        {
            // Construct GraphQl query using raw strings
            let mut graphql_query = "{".to_string();
            for (i, (owner, name)) in gh_ids.iter().enumerate() {
                graphql_query.push_str(&format!(
                    "_{i}: repository(owner: \"{owner}\", name: \"{name}\") {{
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
                                    name
                                }}
                            }}
                        }}
                    }}
                }}"
                ));
            }
            graphql_query.push('}');

            // Send the query
            let response: GraphQlResponse = if !gh_ids.is_empty() {
                GITHUB_API
                    .graphql(&HashMap::from([("query", graphql_query)]))
                    .await?
            } else {
                GraphQlResponse {
                    data: HashMap::new(),
                    errors: Vec::new(),
                }
            };

            errors.extend(response.errors.into_iter().map(|v| {
                (
                    {
                        let id = &gh_ids[v.path[0]
                            .strip_prefix('_')
                            .and_then(|s| s.parse::<usize>().ok())
                            .expect("Unexpected response data")];
                        format!("{}/{}", id.0, id.1)
                    },
                    if v.type_ == "NOT_FOUND" {
                        Error::DoesNotExist
                    } else {
                        Error::GitHubError(v.message)
                    },
                )
            }));

            response
                .data
                .into_values()
                .flatten()
                .map(|d| {
                    (
                        (d.owner.login, d.name),
                        d.releases
                            .nodes
                            .into_iter()
                            .flat_map(|release| {
                                release.release_assets.nodes.into_iter().map(move |asset| {
                                    Metadata {
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
                                    }
                                })
                            })
                            .collect_vec(),
                    )
                })
                .collect_vec()
        };

    let mut success_names = Vec::new();

    for project in cf_projects {
        if let Some(i) = cf_ids.iter().position(|&id| id == project.id) {
            cf_ids.swap_remove(i);
        }

        match curseforge(
            &project,
            profile,
            perform_checks,
            override_profile,
            filters.clone(),
        )
        .await
        {
            Ok(_) => success_names.push(project.name),
            Err(err) => errors.push((format!("{} ({})", project.name, project.id), err)),
        }
    }
    errors.extend(
        cf_ids
            .iter()
            .map(|id| (id.to_string(), Error::DoesNotExist)),
    );

    for project in mr_projects {
        if let Some(i) = mr_ids
            .iter()
            .position(|id| id == &project.id || project.slug.eq_ignore_ascii_case(id))
        {
            mr_ids.swap_remove(i);
        }

        match modrinth(
            &project,
            profile,
            perform_checks,
            override_profile,
            filters.clone(),
        )
        .await
        {
            Ok(_) => success_names.push(project.title),
            Err(err) => errors.push((format!("{} ({})", project.title, project.id), err)),
        }
    }
    errors.extend(
        mr_ids
            .iter()
            .map(|id| (id.to_string(), Error::DoesNotExist)),
    );

    for (repo, asset_names) in gh_repos {
        match github(
            &repo,
            profile,
            Some(asset_names),
            override_profile,
            filters.clone(),
        )
        .await
        {
            Ok(_) => success_names.push(format!("{}/{}", repo.0, repo.1)),
            Err(err) => errors.push((format!("{}/{}", repo.0, repo.1), err)),
        }
    }

    Ok((success_names, errors))
}

/// Check if the repo of `repo_handler` exists, releases mods, and is compatible with `profile`.
/// If so, add it to the `profile`.
///
/// Returns the name of the repository to display to the user
pub async fn github(
    id: &(impl AsRef<str> + ToString, impl AsRef<str> + ToString),
    profile: &mut Profile,
    perform_checks: Option<Vec<Metadata>>,
    override_profile: bool,
    filters: Vec<Filter>,
) -> Result<()> {
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.eq_ignore_ascii_case(id.1.as_ref())
            || matches!(
                &mod_.identifier,
                ModIdentifier::GitHubRepository(owner, repo) if owner == id.0.as_ref() && repo == id.1.as_ref(),
            )
    }) {
        return Err(Error::AlreadyAdded);
    }

    if let Some(download_files) = perform_checks {
        // Check if the repo is compatible
        check::select_latest(
            download_files.iter(),
            if override_profile {
                profile.filters.clone()
            } else {
                [profile.filters.clone(), filters.clone()].concat()
            },
        )
        .await?;
    }

    // Add it to the profile
    profile.push_mod(
        id.1.as_ref().trim().to_string(),
        ModIdentifier::GitHubRepository(id.0.to_string(), id.1.to_string()),
        id.1.as_ref().trim().to_string(),
        override_profile,
        filters,
    );

    Ok(())
}

use ferinth::structures::project::{Project, ProjectType};

/// Check if the project of `project_id` has not already been added, is a mod, and is compatible with `profile`.
/// If so, add it to the `profile`.
pub async fn modrinth(
    project: &Project,
    profile: &mut Profile,
    perform_checks: bool,
    override_profile: bool,
    filters: Vec<Filter>,
) -> Result<()> {
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.eq_ignore_ascii_case(&project.title)
            || matches!(
                &mod_.identifier,
                ModIdentifier::ModrinthProject(id) if id == &project.id,
            )
    }) {
        Err(Error::AlreadyAdded)

    // Check if the project is a mod
    } else if project.project_type != ProjectType::Mod {
        Err(Error::NotAMod)

    // Check if the project is compatible
    } else {
        if perform_checks {
            check::select_latest(
                [Metadata {
                    filename: "".to_owned(),
                    title: "".to_owned(),
                    description: "".to_owned(),
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
                    [profile.filters.clone(), filters.clone()].concat()
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
        // Add it to the profile
        profile.push_mod(
            project.title.trim().to_owned(),
            ModIdentifier::ModrinthProject(project.id.clone()),
            project.slug.to_owned(),
            override_profile,
            filters,
        );
        Ok(())
    }
}

/// Check if the mod of `project_id` has not already been added, is a mod, and is compatible with `profile`.
/// If so, add it to the `profile`.
pub async fn curseforge(
    project: &furse::structures::mod_structs::Mod,
    profile: &mut Profile,
    perform_checks: bool,
    override_profile: bool,
    filters: Vec<Filter>,
) -> Result<()> {
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.eq_ignore_ascii_case(&project.name)
            || matches!(mod_.identifier, ModIdentifier::CurseForgeProject(project_id) | ModIdentifier::PinnedCurseForgeProject(project_id, _) if project_id == project.id)
    }) {
        Err(Error::AlreadyAdded)

    // Check if it can be downloaded by third-parties
    } else if Some(false) == project.allow_mod_distribution {
        Err(Error::DistributionDenied)

    // Check if the project is a Minecraft mod
    } else if !project.links.website_url.as_str().contains("mc-mods") {
        Err(Error::NotAMod)

    // Check if the mod is compatible
    } else {
        if perform_checks {
            check::select_latest(
                [Metadata {
                    filename: "".to_owned(),
                    title: "".to_owned(),
                    description: "".to_owned(),
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
                                .and_then(|l| ModLoader::from_str(&format!("{:?}", l)).ok())
                        })
                        .collect_vec(),
                    channel: ReleaseChannel::Release,
                }]
                .iter(),
                if override_profile {
                    profile.filters.clone()
                } else {
                    [profile.filters.clone(), filters.clone()].concat()
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
            ModIdentifier::CurseForgeProject(project.id),
            project.slug.clone(),
            override_profile,
            filters,
        );

        Ok(())
    }
}
