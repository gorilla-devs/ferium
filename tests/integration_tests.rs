//! All the tests have letters in them so that they run in a correct order
//! Most tests rely on the previous test so they have to run in a correct order
//! This is also why `make test` run cargo test as single threaded, this is to that the tests run sequentially

mod util;
use std::io::Result;
use util::run_command;

#[test]
fn a_argparse() -> Result<()> {
	// Check that arg parsing works
	run_command(vec!["help"])?;
	run_command(vec!["profile", "help"])?;
	// This should create a config file and then fail because there are no profiles
	assert!(run_command(vec!["profile", "list"]).is_err());

	Ok(())
}

#[test]
fn b_create_profile() -> Result<()> {
	// This should succeed
	run_command(vec![
		"profile",
		"create",
		"--name",
		"Fabric 1.18.1",
		"--game-version",
		"1.18.1",
		"--mod-loader",
		"fabric",
		"--output-dir",
		&format!("{}/tests/mods/", env!("PWD")),
	])
}

#[test]
fn b_create_profile_non_existent_game_version() {
	// This should fail because '1.12.3' does not exist
	assert!(run_command(vec![
		"profile",
		"create",
		"--name",
		"Fabric 1.12.3",
		"--game-version",
		"1.12.3",
		"--mod-loader",
		"fabric",
		"--output-dir",
		"/Users/username/mods"
	])
	.is_err());
}

#[test]
fn b_create_profile_output_dir_not_absolute() {
	// This should fail due to the output directory not being absolute
	assert!(run_command(vec![
		"profile",
		"create",
		"--name",
		"Fabric 1.18.1",
		"--game-version",
		"1.18.1",
		"--mod-loader",
		"fabric",
		"--output-dir",
		"mods"
	])
	.is_err());
}
#[test]
fn b_create_profile_missing_args() {
	// This should fail due to missing arguments
	assert!(run_command(vec![
		"profile",
		"create",
		"--game-version",
		"1.18.1",
		"--mod-loader",
		"fabric",
		"--output-dir",
		"/Users/username/mods"
	])
	.is_err());
}
#[test]
fn c_create_profile_name_already_exists() {
	// This should fail because a profile with the same name already exists
	assert!(run_command(vec![
		"profile",
		"create",
		"--name",
		"Fabric 1.18.1",
		"--game-version",
		"1.18.1",
		"--mod-loader",
		"fabric",
		"--output-dir",
		"/Users/username/mods",
	])
	.is_err());
}

#[test]
fn d_add_modrinth() -> Result<()> {
	// Add Sodium to config
	run_command(vec!["add-modrinth", "starlight"])?;
	// Check that trying to add the same mod again fails
	assert!(run_command(vec!["add-modrinth", "StArLiGhT"]).is_err());

	Ok(())
}

#[test]
fn d_add_curseforge() -> Result<()> {
	// Add Terralith to the config
	run_command(vec!["add-curseforge", "513688"])?;
	// Check that trying to add the same mod again fails
	assert!(run_command(vec!["add-curseforge", "513688"]).is_err());

	Ok(())
}

#[test]
fn d_add_github() -> Result<()> {
	// Add Sodium to config
	run_command(vec!["add-github", "CaffeineMC", "sodium-fabric"])?;
	// Check that trying to add the same repo again fails
	assert!(run_command(vec!["add-github", "caffeinemc", "Sodium-Fabric"]).is_err());

	Ok(())
}

#[test]
fn e_list() -> Result<()> {
	run_command(vec!["list"])
}

#[test]
fn e_list_verbose() -> Result<()> {
	run_command(vec!["list", "--verbose"])?;
	run_command(vec!["list", "-v"])
}

#[test]
fn e_profile_list() -> Result<()> {
	run_command(vec!["profile", "list"])
}

#[test]
fn f_upgrade() -> Result<()> {
	// TODO: Check that downloaded mods are present in the output_dir
	run_command(vec!["upgrade", "--no-picker"])?;
	run_command(vec!["upgrade", "--no-picker", "--no-patch-check"])
}

#[test]
fn g_switch() -> Result<()> {
	// Add an additional forge profile
	run_command(vec![
		"profile",
		"create",
		"--name",
		"Forge 1.12.2",
		"--game-version",
		"1.12.2",
		"--mod-loader",
		"forge",
		"--output-dir",
		"/Users/username/mods",
	])?;

	// Check that listing mods gives an error (no mods/repos in new profile)
	assert!(run_command(vec!["list"]).is_err());
	// Add Sodium to config for the delete profile test
	run_command(vec!["add-modrinth", "sodium"])?;

	// Switch to the fabric profile and check that it contains the mods added before
	run_command(vec!["switch", "--profile-name", "Fabric 1.18.1"])?;
	run_command(vec!["list"])
}

#[test]
fn h_remove() -> Result<()> {
	// These should fail as one of the mod names provided does not exist
	assert!(run_command(vec![
		"remove",
		"--mod-names",
		"starlght", // Wrong
		"--mod-names",
		"terralith",
		"--mod-names",
		"sodium",
	])
	.is_err());
	assert!(run_command(vec![
		"remove",
		"--mod-names",
		"starlight",
		"--mod-names",
		"terrlith", // Wrong
		"--mod-names",
		"sodium",
	])
	.is_err());
	assert!(run_command(vec![
		"remove",
		"--mod-names",
		"starlight",
		"--mod-names",
		"terralith",
		"--mod-names",
		"sodum", // Wrong
	])
	.is_err());

	// Remove all the mods
	run_command(vec![
		"remove",
		"--mod-names",
		"starlight",
		"--mod-names",
		"terralith",
		"--mod-names",
		"sodium-fabric",
	])?;
	// Check that listing mods gives an error (because there are no mods)
	assert!(run_command(vec!["list"]).is_err());

	Ok(())
}

#[test]
fn i_delete_profile_index_correction() -> Result<()> {
	// Remove the fabric profile
	run_command(vec!["profile", "delete", "--profile-name", "Fabric 1.18.1"])?;
	// Ferium should have switched to the forge profile, so listing should show the sodium mod (which was added in the switch test)
	run_command(vec!["list"])
}
#[test]
fn j_delete_all_profile() -> Result<()> {
	// Remove the forge profile
	run_command(vec!["profile", "delete", "--profile-name", "Forge 1.12.2"])?;
	// Listing profiles should result in an error
	assert!(run_command(vec!["profile list"]).is_err());

	Ok(())
}
