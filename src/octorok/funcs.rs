/*
 * This file contains abstractions of some of the calls supported by the GitHub API
 */

use super::structs::*;
use bytes::Bytes;
use reqwest::{header::USER_AGENT, Client, Response};
use std::process::exit;

/// Returns the contents of `release`'s JAR file as bytes
pub async fn download_release(client: &Client, release: &Release) -> Bytes {
    let mut contents: Option<Bytes> = None;

    // For each asset
    for asset in &release.assets {
        // If it is a JAR file
        if asset.content_type == "application/java-archive" {
            // Download the file and store it to `contents`
            contents = match request(client, &asset.browser_download_url, false)
                .await
                .bytes()
                .await
            {
                Ok(c) => Some(c),
                Err(e) => {
                    println!("No response from server. {}", e);
                    exit(124);
                }
            }
        }
    }

    match contents {
        // If contents is not null, return it
        Some(c) => c,
        // Otherwise, the release doesn't have JAR assets
        None => {
            println!("Could not find JAR asset!");
            exit(124);
        }
    }
}

/// Returns the repository at https://github.com/{owner}/{repo}
pub async fn get_repository(client: &Client, owner: &str, repo: &str) -> Repository {
    match request(client, &format!("/repos/{}/{}", owner, repo), true)
        .await
        .json()
        .await
    {
        Ok(repo) => repo,
        Err(e) => {
            println!("JSON deserialisation failed due to {}", e);
            exit(122);
        }
    }
}

/// Returns the releases in `repo` sorted in chronologically descending order
pub async fn get_releases(client: &Client, repo: Repository) -> Vec<Release> {
    match request(client, &format!("/repos/{}/releases", repo.full_name), true)
        .await
        .json()
        .await
    {
        Ok(release) => release,
        Err(e) => {
            println!("JSON deserialisation failed due to {}", e);
            exit(122);
        }
    }
}

/// Send a request to `url` and return result. If `realitive` is true, the Labrinth base url is prepended
pub async fn request(client: &Client, url: &str, relative: bool) -> Response {
    let url: String = if relative {
        // If provided URL is specified as relative, then prepend the base url
        format!("http://api.github.com{}", url).into()
    } else {
        // Else directly assign the URL
        url.into()
    };

    // User agent header required by GitHub
    let request = client.get(&url).header(USER_AGENT, "ferium");

    match request.send().await {
        Ok(response) => {
            if response.status().is_success() {
                response
            } else {
                println!("HTTP request failed with error code {}.", response.status());
                exit(124);
            }
        }
        Err(e) => {
            println!("HTTP request failed due to {}", e);
            exit(124);
        }
    }
}
