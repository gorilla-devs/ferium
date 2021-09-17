//! This file contains abstractions of some of the calls supported by the GitHub API

use super::structs::*;
use bytes::Bytes;
use reqwest::{header::USER_AGENT, Client, Response};

/// Returns the contents of `release`'s JAR file as bytes
pub async fn download_release(client: &Client, release: &Release) -> Bytes {
    let mut contents: Option<Bytes> = None;

    // For each asset
    for asset in &release.assets {
        // If it is a JAR file
        if asset.content_type.contains("java") {
            // Download the file and store it to `contents`
            contents = Some(
                request(client, asset.browser_download_url.clone())
                    .await
                    .bytes()
                    .await
                    .unwrap(),
            )
        }
    }

    match contents {
        // If contents is not null, return it
        Some(c) => c,
        // Otherwise, the release doesn't have JAR assets
        None => {
            panic!("Could not find JAR asset!");
        }
    }
}

/// Returns the repository at https://github.com/{owner}/{repo}
pub async fn get_repository(client: &Client, owner: &str, repo: &str) -> Repository {
    request_rel(client, format!("/repos/{}/{}", owner, repo))
        .await
        .json()
        .await
        .unwrap()
}

/// Returns the releases in `repo` sorted in chronologically descending order
pub async fn get_releases(client: &Client, repo: Repository) -> Vec<Release> {
    request_rel(client, format!("/repos/{}/releases", repo.full_name))
        .await
        .json()
        .await
        .unwrap()
}

/// Send a request to `url` with `client` and return response. The GitHub API's base URL will be prepended to `url`
pub async fn request_rel(client: &Client, url: String) -> Response {
    request(client, format!("http://api.github.com{}", url).into()).await
}

/// Send a request to `url` with `client` and return response
pub async fn request(client: &Client, url: String) -> Response {
    // User agent header required by GitHub
    let request = client.get(&url).header(USER_AGENT, "ferium");

    let response = request.send().await.unwrap();
    if response.status().is_success() {
        response
    } else {
        panic!("HTTP request failed with error code {}", response.status());
    }
}
