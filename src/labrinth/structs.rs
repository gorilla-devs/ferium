//! This file contains typed structs of the data structures used by the Labrinth API
//! These are copied from https://github.com/modrinth/labrinth/wiki/API-Documentation#structure-definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::vec::Vec;

/// An RFC3339 formatted datetime
pub type Datetime = String;
/// A random base 62 number. These are stored as strings in JSON.
pub type ID = String;

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    /// The user's ID
    pub id: ID,
    /// The user's Github ID. Only visible to the user themselves
    pub github_id: Option<u64>,
    /// The user's username
    pub username: String,
    /// The user's display name
    pub name: String,
    /// The user's email Only visible to the user themselves
    pub email: Option<String>,
    /// The user's avatar url. Uses Github's icons
    pub avatar_url: Option<String>,
    /// A description of the user
    pub bio: String,
    /// The time at which the user was created
    pub created: Datetime,
    /// The user's role - developer, moderator, or admin
    pub role: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Mod {
    /// The ID of the mod, encoded as a base62 string
    pub id: ID,
    /// The slug of a mod, typically a snake case of the mod name
    pub slug: String,
    /// The ID of the team that has ownership of this mod
    pub team: ID,
    /// The title or name of the mod
    pub title: String,
    /// A short description of the mod
    pub description: String,
    /// A long form of the description
    pub body: String,
    #[deprecated(note = "Read from `Mod.body` instead")]
    /// The link to the long description of the mod
    pub body_url: Option<String>,
    /// The date at which the mod was first published
    pub published: Datetime,
    /// The date at which the mod was last updated
    pub updated: Datetime,
    /// The status of the mod - approved, rejected, draft, unlisted, processing, or unknown
    pub status: String,
    /// The license of the mod
    pub license: License,
    /// The support range for the client mod - required, optional, unsupported, or unknown
    pub client_side: String,
    /// The support range for the server mod - required, optional, unsupported, or unknown
    pub server_side: String,
    /// The total number of downloads the mod has
    pub downloads: usize,
    /// A list of the categories that the mod is in
    pub categories: Vec<String>,
    /// A list of IDs for versions of the mod
    pub versions: Vec<ID>,
    /// The link to the mod's icon
    pub icon_url: Option<String>,
    /// A link on where to submit bugs or issues with the mod
    pub issues_url: Option<String>,
    /// A link to the source code for the mod
    pub source_url: Option<String>,
    /// A link to the mod's wiki page or other relevant information
    pub wiki_url: Option<String>,
    /// A link to the mod's discord
    pub discord_url: Option<String>,
    /// A list of all donation links the mod has
    pub donation_urls: Option<Vec<DonationLink>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Version {
    /// The ID of the version, encoded as a base62 string
    pub id: ID,
    /// The ID of the mod this version is for
    pub mod_id: ID,
    /// The ID of the author who published this version
    pub author_id: ID,
    /// Whether the version is featured or not
    pub featured: bool,
    /// The name of this version
    pub name: String,
    /// The version number. Ideally, this will follow semantic versioning
    pub version_number: String,
    /// The changelog for this version of the mod.
    pub changelog: Option<String>,
    #[deprecated(note = "Read from `Version.changelog` instead")]
    /// A link to the changelog for this version of the mod
    pub changelog_url: Option<String>,
    /// The date that this version was published
    pub date_published: Datetime,
    /// The number of downloads this version has
    pub downloads: usize,
    /// The type of the release - alpha, beta, or release
    pub version_type: String,
    /// A list of files available for download
    pub files: Vec<VersionFile>,
    /// This version's dependencies, as a list of IDs of dependency's version
    pub dependencies: Vec<ID>,
    /// A list of versions of Minecraft that this version of the mod supports
    pub game_versions: Vec<String>,
    /// The mod loaders that this version supports
    pub loaders: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
/// A single mod file, with a URL for the file and the file's hashes
pub struct VersionFile {
    /// The key is the hashing algorithm and the value is the hash
    pub hashes: HashMap<String, String>,
    /// A direct link to the file
    pub url: String,
    /// The name of the file
    pub filename: String,
}

#[derive(Deserialize, Serialize, Debug)]
/// The license of a mod, representing the short id, long form name, and a URL
pub struct License {
    /// The license ID of a mod, retrieved from the license's get route
    pub id: String,
    /// The long name of the license
    pub name: String,
    /// The URL to this license
    pub url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
/// A donation link of a mod, representing the platform's id, platform name, and a URL
pub struct DonationLink {
    /// The platform ID of a mod, retrieved from the donation platform's get route
    pub id: String,
    /// The long name of the platform
    pub platform: String,
    /// The URL to this donation link
    pub url: String,
}
