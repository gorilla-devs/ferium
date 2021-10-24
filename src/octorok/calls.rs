//! This file contains abstractions of some of the calls supported by the GitHub API

use super::structs::*;
use crate::ferium_error::{FError, FResult};
use bytes::Bytes;
use reqwest::{header::USER_AGENT, Client, Response};

/// Returns the contents of `release`'s JAR file as bytes
pub async fn download_release(client: &Client, release: &Release) -> FResult<Bytes> {
    let mut contents: Option<Bytes> = None;

    // For each asset
    for asset in &release.assets {
        // If it is a JAR file
        if asset.name.contains(".jar") {
            // Download the file and store it to `contents`
            contents = Some(
                request(client, asset.browser_download_url.clone())
                    .await?
                    .bytes()
                    .await?,
            )
        }
    }

    match contents {
        // If contents is not null, return it
        Some(c) => Ok(c),
        // Otherwise, the release doesn't have JAR assets
        None => Err(FError::Quit {
            message: "Could not find JAR asset!".into(),
        }),
    }
}

/// Returns the repository at https://github.com/{owner}/{repo}
pub async fn get_repository(client: &Client, owner: &str, repo: &str) -> FResult<Repository> {
    Ok(request_rel(client, format!("/repos/{}/{}", owner, repo))
        .await?
        .json()
        .await?)
}

/// Returns the releases in `repo` sorted in chronologically descending order
pub async fn get_releases(client: &Client, repo: &Repository) -> FResult<Vec<Release>> {
    Ok(
        request_rel(client, format!("/repos/{}/releases", repo.full_name))
            .await?
            .json()
            .await?,
    )
}

/// Send a request to `url` with `client` and return response. Labrinth's base URL will be prepended to `url`
async fn request_rel(client: &Client, url: String) -> FResult<Response> {
    Ok(request(client, format!("http://api.github.com{}", url)).await?)
}

/// Send a request to `url` with `client` and return response
async fn request(client: &Client, url: String) -> FResult<Response> {
    // User agent header required by GitHub
    let request = client.get(url).header(USER_AGENT, "ferium");

    let response = request.send().await?;
    if response.status().is_success() {
        Ok(response)
    } else {
        Err(FError::HTTPError {
            message: format!("HTTP request failed with error code {}", response.status()),
        })
    }
}
