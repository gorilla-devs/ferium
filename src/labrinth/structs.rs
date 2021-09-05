/*
 * This file contains typed structs of the data structures used by the Labrinth API
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::vec::Vec;

pub type Datetime = String; // An RFC3339 formatted datetime
pub type ID = String; // A random base 62 number. These are stored as strings in JSON.

#[derive(Deserialize, Serialize, Debug)]
pub struct Version {
    pub id: ID,
    pub mod_id: ID,
    pub author_id: ID,
    pub featured: bool,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub changelog_url: Option<String>,
    pub date_published: Datetime,
    pub downloads: isize,
    pub version_type: String,
    pub files: Vec<VersionFile>,
    pub dependencies: Vec<ID>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VersionFile {
    pub hashes: HashMap<String, String>,
    pub url: String,
    pub filename: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub id: ID,
    pub github_id: Option<u64>,
    pub username: String,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: String,
    pub created: Datetime,
    pub role: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Mod {
    pub id: ID,
    pub slug: String,
    pub team: ID,
    pub title: String,
    pub description: String,
    pub body: String,
    pub body_url: Option<String>,
    pub published: Datetime,
    pub updated: Datetime,
    pub status: String,
    pub license: License,
    pub client_side: String,
    pub server_side: String,
    pub downloads: isize,
    pub categories: Vec<String>,
    pub versions: Vec<ID>,
    pub icon_url: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub donation_urls: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct License {
    pub id: String,
    pub name: String,
    pub url: String,
}
