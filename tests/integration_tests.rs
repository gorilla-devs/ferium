mod util;

use libium::HOME;
use std::fs::remove_dir;
use util::run_command;

type Result = std::io::Result<()>;

#[test]
fn argparse() -> Result {
    run_command(vec!["help"], None)?;
    run_command(vec!["profile", "help"], None)
}

#[test]
fn create_config_file_when_none() {
    // This should create a config file and then fail because there are no profiles
    assert!(run_command(vec!["profile", "list"], None).is_err());
}

#[test]
fn create_profile() -> Result {
    let mut home = HOME.clone();
    home.push("mods");
    run_command(
        vec![
            "profile",
            "create",
            "--name",
            "Test profile",
            "--game-version",
            "1.12.2",
            "--mod-loader",
            "forge",
            "--output-dir",
            home.to_str().unwrap(),
        ],
        Some("empty"),
    )
}

#[test]
fn create_profile_import_mods() -> Result {
    let mut home = HOME.clone();
    home.push("mods");
    run_command(
        vec![
            "profile",
            "create",
            "--name",
            "Test profile",
            "--game-version",
            "1.12.2",
            "--mod-loader",
            "forge",
            "--output-dir",
            home.to_str().unwrap(),
            "--import",
            "Default Modded",
        ],
        Some("one_profile_full"),
    )
}

#[test]
fn create_profile_output_dir_not_absolute() {
    assert!(run_command(
        vec![
            "profile",
            "create",
            "--name",
            "Test profile",
            "--game-version",
            "1.12.2",
            "--mod-loader",
            "forge",
            "--output-dir",
            "mods" // oops relative directory
        ],
        Some("empty")
    )
    .is_err());
}

#[test]
fn create_profile_missing_args() {
    let mut home = HOME.clone();
    home.push("mods");
    assert!(run_command(
        vec![
            "profile",
            "create",
            // oops no name
            "--game-version",
            "1.12.2",
            "--mod-loader",
            "forge",
            "--output-dir",
            home.to_str().unwrap(),
        ],
        Some("empty")
    )
    .is_err());
}

#[test]
fn create_profile_name_already_exists() {
    let mut home = HOME.clone();
    home.push("mods");
    assert!(run_command(
        vec![
            "profile",
            "create",
            "--name",
            "Test profile", // oops profile with same name already exists
            "--game-version",
            "1.12.2",
            "--mod-loader",
            "forge",
            "--output-dir",
            home.to_str().unwrap(),
        ],
        Some("empty_profile")
    )
    .is_err());
}

#[test]
fn add_modrinth() -> Result {
    // Add Sodium to config
    run_command(vec!["add-modrinth", "starlight"], Some("empty_profile"))
}

#[test]
fn add_curseforge() -> Result {
    // Add Terralith to the config
    run_command(vec!["add-curseforge", "591388"], Some("empty_profile"))
}

#[test]
fn add_github() -> Result {
    // Add Sodium to config
    run_command(
        vec!["add-github", "CaffeineMC", "sodium-fabric"],
        Some("empty_profile"),
    )
}

#[test]
fn already_added() {
    assert!(run_command(vec!["add-modrinth", "StArLiGhT"], Some("one_profile_full")).is_err());
    assert!(run_command(vec!["add-curseforge", "591388"], Some("one_profile_full")).is_err());
    assert!(run_command(
        vec!["add-github", "caffeinemc", "Sodium-Fabric"],
        Some("one_profile_full")
    )
    .is_err());
}

#[test]
fn list() -> Result {
    run_command(vec!["list"], Some("one_profile_full"))?;
    run_command(vec!["list", "--verbose"], Some("one_profile_full"))
}

#[test]
fn profile_list() -> Result {
    run_command(vec!["profile", "list"], Some("one_profile_full"))
}

#[test]
fn upgrade() -> Result {
    let _ = remove_dir("./tests/mods");
    run_command(vec!["upgrade"], Some("one_profile_full"))
}

#[test]
fn switch() -> Result {
    // Switch to the fabric profile and check that it contains the mods added before
    run_command(
        vec!["switch", "--profile-name", "Profile Two"],
        Some("two_profiles_one_empty"),
    )?;
    run_command(vec!["list"], Some("two_profiles_one_empty"))
}

#[test]
fn remove_fail() {
    // These should fail as one of the mod names provided does not exist
    assert!(run_command(
        vec![
            "remove",
            "starlght", // Wrong
            "incendium",
            "sodium",
        ],
        Some("one_profile_full")
    )
    .is_err());
    assert!(run_command(
        vec![
            "remove",
            "starlight (fabric)",
            "incendum", // Wrong
            "sodium",
        ],
        Some("one_profile_full")
    )
    .is_err());
    assert!(run_command(
        vec![
            "remove",
            "starlight (fabric)",
            "incendium",
            "sodum", // Wrong
        ],
        Some("one_profile_full")
    )
    .is_err());
}

#[test]
fn remove_all() -> Result {
    run_command(
        vec!["remove", "starlight (fabric)", "incendium", "sodium-fabric"],
        Some("one_profile_full"),
    )
}

#[test]
fn delete_profile() -> Result {
    run_command(
        vec!["profile", "delete", "--profile-name", "Profile Two"],
        Some("two_profiles_one_empty"),
    )
}
