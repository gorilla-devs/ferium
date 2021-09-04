use super::structs::*;
use bytes::Bytes;
use reqwest::blocking::{get, Response};

/// Return the contents of `version`'s JAR file as bytes
pub fn download_version(version: &Version) -> Bytes {
    match request(&version.files[0].url, false).bytes() {
        Ok(contents) => contents,
        Err(e) => panic!("No response from server. {}", e),
    }
}

/// Returns the versions of `mod_id`'s mod sorted in chronologically descending order
pub fn get_versions(mod_id: &str) -> Vec<Version> {
    match request(&format!("/mod/{}/version", mod_id), true).json() {
        Ok(typed) => typed,
        Err(e) => panic!("JSON deserialisation failed due to {}", e),
    }
}

/// Get a mod using the `mod_slug`, which can also be the mod ID
pub fn get_mod(mod_slug: &str) -> Mod {
    match request(&format!("/mod/{}", mod_slug), true).json() {
        Ok(typed) => typed,
        Err(e) => panic!("JSON deserialisation failed due to {}", e),
    }
}

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
                panic!("HTTP request failed with error code {}", response.status());
            }
        }
        Err(e) => panic!("HTTP request failed due to {}", e),
    }
}
