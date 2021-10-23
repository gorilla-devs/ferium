//! This file contains abstractions of some of the calls supported by the Labrinth API

use super::structs::*;
use crate::ferium_error::{FError, FResult};
use bytes::Bytes;
use reqwest::{Client, Response};

/// Return the contents of `version`'s JAR file as bytes
pub async fn download_version(client: &Client, version: Version) -> FResult<Bytes> {
    Ok(request(client, version.files[0].url.clone())
        .await?
        .bytes()
        .await?)
}

/// Returns the versions of `mod_id`'s mod sorted in chronologically descending order
pub async fn get_versions(client: &Client, mod_id: &str) -> FResult<Vec<Version>> {
    Ok(request_rel(client, format!("/mod/{}/version", mod_id))
        .await?
        .json()
        .await?)
}

/// Get a mod using the `mod_slug`, which can also be the mod ID
pub async fn get_mod(client: &Client, mod_slug: &str) -> FResult<Mod> {
    Ok(request_rel(client, format!("/mod/{}", mod_slug))
        .await?
        .json()
        .await?)
}

/// Send a request to `url` with `client` and return response. Labrinth's base URL will be prepended to `url`
async fn request_rel(client: &Client, url: String) -> FResult<Response> {
    Ok(request(client, format!("https://api.modrinth.com/api/v1{}", url)).await?)
}

/// Send a request to `url` with `client` and return response
async fn request(client: &Client, url: String) -> FResult<Response> {
    let response = client.get(url).send().await?;
    if response.status().is_success() {
        Ok(response)
    } else {
        Err(FError::HTTPError {
            message: format!("HTTP request failed with error code {}", response.status()),
        })
    }
}
