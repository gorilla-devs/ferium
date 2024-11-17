mod util;

use libium::HOME;
use std::fs::remove_dir;
use util::run_command;

type Result = std::io::Result<()>;

fn output_dir() -> String {
    let mut home = HOME.clone();
    home.push("mods");
    home.to_string_lossy().to_string()
}

#[test]
fn argparse() -> Result {
    run_command(vec!["help"], None)?;
    run_command(vec!["profile", "help"], None)
}

#[test]
fn create_profile() -> Result {
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
            &output_dir(),
        ],
        // This also tests that the config file is automatically created
        None,
    )
}

#[test]
fn create_profile_import_mods() -> Result {
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
            &output_dir(),
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
            &output_dir(),
        ],
        Some("empty")
    )
    .is_err());
}

#[test]
fn create_profile_name_already_exists() {
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
            &output_dir(),
        ],
        Some("empty_profile")
    )
    .is_err());
}

#[test]
fn add_modrinth() -> Result {
    // Add Sodium to config
    run_command(vec!["add", "starlight"], Some("empty_profile"))
}

#[test]
fn add_curseforge() -> Result {
    // Add Terralith to the config
    run_command(vec!["add", "591388"], Some("empty_profile"))
}

#[test]
fn add_github() -> Result {
    // Add Sodium to config
    run_command(vec!["add", "CaffeineMC/sodium"], Some("empty_profile"))
}

#[test]
fn add_all() -> Result {
    run_command(
        vec!["add", "starlight", "591388", "CaffeineMC/sodium"],
        Some("empty_profile"),
    )
}

#[test]
fn scan_dir() -> Result {
    run_command(
        vec!["scan", "--directory", "./tests/test_mods"],
        Some("empty_profile"),
    )
}

#[test]
fn modpack_add_modrinth() -> Result {
    // Add Fabulously Optimised
    run_command(
        vec![
            "modpack",
            "add",
            "1KVo5zza",
            "--output-dir",
            &output_dir(),
            "--install-overrides",
            "true",
        ],
        Some("empty_profile"),
    )
}

#[test]
fn modpack_add_curseforge() -> Result {
    // Add RLCraft
    run_command(
        vec![
            "modpack",
            "add",
            "452013",
            "--output-dir",
            &output_dir(),
            "--install-overrides",
            "true",
        ],
        Some("empty_profile"),
    )
}

#[test]
fn already_added() {
    assert!(run_command(vec!["add", "StArLiGhT"], Some("one_profile_full")).is_ok());
    assert!(run_command(vec!["add", "591388"], Some("one_profile_full")).is_ok());
    assert!(run_command(vec!["add", "cAfFeInEmC/SoDiUm"], Some("one_profile_full")).is_ok());
}

#[test]
fn list() -> Result {
    run_command(vec!["list"], Some("one_profile_full"))
}

#[test]
fn list_verbose() -> Result {
    run_command(vec!["list", "--verbose"], Some("one_profile_full"))
}

#[test]
fn list_markdown() -> Result {
    run_command(
        vec!["list", "--verbose", "--markdown"],
        Some("one_profile_full"),
    )
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
fn cf_modpack_upgrade() -> Result {
    let _ = remove_dir("./tests/cf_modpack");
    run_command(vec!["modpack", "upgrade"], Some("two_modpacks_cfactive"))
}

#[test]
fn md_modpack_upgrade() -> Result {
    let _ = remove_dir("./tests/md_modpack");
    run_command(vec!["modpack", "upgrade"], Some("two_modpacks_mdactive"))
}

#[test]
fn profile_switch() -> Result {
    run_command(
        vec!["profile", "switch", "Profile Two"],
        Some("two_profiles_one_empty"),
    )
}

#[test]
fn modpack_switch() -> Result {
    run_command(
        vec!["modpack", "switch", "CF Fabulously Optimised"],
        Some("two_modpacks_mdactive"),
    )
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
fn remove_name() -> Result {
    run_command(
        vec!["remove", "starlight (fabric)", "incendium", "sodium"],
        Some("one_profile_full"),
    )
}

#[test]
fn remove_id() -> Result {
    run_command(
        vec!["remove", "H8CaAYZC", "591388", "caffeinemc/sodium"],
        Some("one_profile_full"),
    )
}

#[test]
fn delete_profile() -> Result {
    run_command(
        vec!["profile", "delete", "Profile Two"],
        Some("two_profiles_one_empty"),
    )
}

#[test]
fn delete_modpack() -> Result {
    run_command(
        vec!["modpack", "delete", "CF Fabulously Optimised"],
        Some("two_modpacks_mdactive"),
    )
}
