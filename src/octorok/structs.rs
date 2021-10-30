//! This file contains typed structs of the data structures used by the GitHub API

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    pub id: usize,
    pub node_id: String,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub owner: User,
    pub html_url: String,
    pub description: String,
    pub fork: bool,
    pub url: String,
    pub forks_url: String,
    pub keys_url: String,
    pub collaborators_url: String,
    pub teams_url: String,
    pub hooks_url: String,
    pub issue_events_url: String,
    pub events_url: String,
    pub assignees_url: String,
    pub branches_url: String,
    pub tags_url: String,
    pub blobs_url: String,
    pub git_tags_url: String,
    pub git_refs_url: String,
    pub trees_url: String,
    pub statuses_url: String,
    pub languages_url: String,
    pub stargazers_url: String,
    pub contributors_url: String,
    pub subscribers_url: String,
    pub subscription_url: String,
    pub commits_url: String,
    pub git_commits_url: String,
    pub comments_url: String,
    pub issue_comment_url: String,
    pub contents_url: String,
    pub compare_url: String,
    pub merges_url: String,
    pub archive_url: String,
    pub downloads_url: String,
    pub issues_url: String,
    pub pulls_url: String,
    pub milestones_url: String,
    pub notifications_url: String,
    pub labels_url: String,
    pub releases_url: String,
    pub deployments_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub pushed_at: String,
    pub git_url: String,
    pub ssh_url: String,
    pub clone_url: String,
    pub svn_url: String,
    pub homepage: String,
    pub size: usize,
    pub stargazers_count: usize,
    pub watchers_count: usize,
    pub language: String,
    pub has_issues: bool,
    pub has_projects: bool,
    pub has_downloads: bool,
    pub has_wiki: bool,
    pub has_pages: bool,
    pub forks_count: usize,
    pub mirror_url: Option<String>,
    pub archived: bool,
    pub disabled: bool,
    pub open_issues_count: usize,
    pub license: License,
    pub forks: usize,
    pub open_issues: usize,
    pub watchers: usize,
    pub default_branch: String,
    pub temp_clone_token: Option<String>,
    pub organization: Option<User>,
    pub network_count: usize,
    pub subscribers_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Release {
    pub url: String,
    pub assets_url: String,
    pub upload_url: String,
    pub html_url: String,
    pub id: usize,
    pub author: User,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: String,
    pub assets: Vec<Asset>,
    pub tarball_url: String,
    pub zipball_url: String,
    pub body: String,
    pub reactions: Option<Reactions>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub login: String,
    pub id: usize,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub site_admin: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Asset {
    pub url: String,
    pub id: usize,
    pub node_id: String,
    pub name: String,
    pub label: Option<String>,
    pub uploader: User,
    pub content_type: String,
    pub state: String,
    pub size: usize,
    pub download_count: usize,
    pub created_at: String,
    pub updated_at: String,
    pub browser_download_url: String,
}

impl std::fmt::Display for Asset {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.name)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reactions {
    pub url: String,
    pub total_count: usize,
    #[serde(rename = "+1")]
    pub plus_one: usize,
    #[serde(rename = "-1")]
    pub minus_one: usize,
    pub laugh: usize,
    pub hooray: usize,
    pub confused: usize,
    pub heart: usize,
    pub rocket: usize,
    pub eyes: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct License {
    pub key: String,
    pub name: String,
    pub spdx_id: String,
    pub url: Option<String>,
    pub node_id: String,
}
