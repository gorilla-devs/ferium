//! This file contains abstractions of some of the calls supported by the GitHub API

use super::structs::*;
use crate::ferium_error::{FError, FResult};
use bytes::Bytes;
use reqwest::{header::USER_AGENT, Client, Response};

/// Downloads `asset`'s contents and returns them as bytes
pub async fn download_asset(client: &Client, asset: &Asset) -> FResult<Bytes> {
    Ok(request(client, asset.browser_download_url.clone())
        .await?
        .bytes()
        .await?)
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
    Ok(request(client, format!("https://api.github.com{}", url)).await?)
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
