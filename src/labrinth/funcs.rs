/*
 * This file contains abstractions of some of the calls supported by the Labrinth API
 */

use super::structs::*;
use bytes::Bytes;
use reqwest::blocking::{get, Response};
use reqwest::StatusCode;
use std::process::exit;

/// Return the contents of `version`'s JAR file as bytes
pub fn download_version(version: &Version) -> Bytes {
    match request(&version.files[0].url, false).bytes() {
        Ok(contents) => contents,
        Err(e) => {
            println!("No response from server. {}", e);
            exit(124);
        }
    }
}

/// Checks if a mod exists. If it does, then the mod is returned, else None is returned
pub fn does_exist(mod_id: &ID) -> Option<Mod> {
    let response = request(&format!("/mod/{}", mod_id), true);
    match response.status() {
        StatusCode::OK => Some(match response.json() {
            Ok(typed) => typed,
            Err(e) => {
                println!("JSON deserialisation failed due to {}", e);
                exit(122);
            }
        }),
        _ => Option::None,
    }
}

/// Returns the versions of `mod_id`'s mod sorted in chronologically descending order
pub fn get_versions(mod_id: &str) -> Vec<Version> {
    match request(&format!("/mod/{}/version", mod_id), true).json() {
        Ok(typed) => typed,
        Err(e) => {
            println!("JSON deserialisation failed due to {}", e);
            exit(122);
        }
    }
}

/// Get a mod using the `mod_slug`, which can also be the mod ID
pub fn get_mod(mod_slug: &str) -> Mod {
    match request(&format!("/mod/{}", mod_slug), true).json() {
        Ok(typed) => typed,
        Err(e) => {
            println!("JSON deserialisation failed due to {}", e);
            exit(122);
        }
    }
}

/// Send a request to `url` and return result. If `realitive` is true, the Labrinth base url is prepended
fn request(url: &str, relative: bool) -> Response {
    let url: String = if relative {
        // If provided URL is specified as relative, then prepend the base url
        format!("https://api.modrinth.com/api/v1{}", url).into()
    } else {
        // Else directly assign the URL
        url.into()
    };

    match get(url) {
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
