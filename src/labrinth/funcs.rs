/*
 * This file contains abstractions of some of the calls supported by the Labrinth API
 */

use super::structs::*;
use bytes::Bytes;
use reqwest::StatusCode;
use reqwest::{Client, Response};
use std::process::exit;

/// Return the contents of `version`'s JAR file as bytes
pub async fn download_version(client: &Client, version: &Version) -> Bytes {
    match request(client, &version.files[0].url, false)
        .await
        .bytes()
        .await
    {
        Ok(contents) => contents,
        Err(e) => {
            println!("No response from server. {}", e);
            exit(124);
        }
    }
}

/// Checks if a mod exists. If it does, then the mod is returned, else None is returned
pub async fn does_exist(client: &Client, mod_id: &ID) -> Option<Mod> {
    let response = request(client, &format!("/mod/{}", mod_id), true).await;
    match response.status() {
        StatusCode::OK => match response.json().await {
            Ok(typed) => Some(typed),
            Err(e) => {
                println!("JSON deserialisation failed due to {}", e);
                exit(122);
            }
        },
        _ => Option::None,
    }
}

/// Returns the versions of `mod_id`'s mod sorted in chronologically descending order
pub async fn get_versions(client: &Client, mod_id: &str) -> Vec<Version> {
    match request(client, &format!("/mod/{}/version", mod_id), true)
        .await
        .json()
        .await
    {
        Ok(typed) => typed,
        Err(e) => {
            println!("JSON deserialisation failed due to {}", e);
            exit(122);
        }
    }
}

/// Get a mod using the `mod_slug`, which can also be the mod ID
pub async fn get_mod(client: &Client, mod_slug: &str) -> Mod {
    match request(client, &format!("/mod/{}", mod_slug), true)
        .await
        .json()
        .await
    {
        Ok(typed) => typed,
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
        format!("https://api.modrinth.com/api/v1{}", url).into()
    } else {
        // Else directly assign the URL
        url.into()
    };

    match client.get(url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                response
            } else {
                println!("HTTP request failed with error code {}", response.status());
                exit(124);
            }
        }
        Err(e) => {
            println!("HTTP request failed due to {}", e);
            exit(124);
        }
    }
}
