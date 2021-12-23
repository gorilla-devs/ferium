use super::*;
use util::json::Profile;
use util::wrappers::{get_latest_mc_versions, remove_semver_patch};

fn s() -> String {
	String::new()
}

fn p() -> PathBuf {
	PathBuf::new()
}

fn v<T>() -> Vec<T> {
	Vec::new()
}

#[test]
fn test_check_empty_config() {
	assert!(check_empty_profile(&Profile {
		name: s(),
		output_dir: p(),
		game_version: s(),
		mod_loader: s(),
		mod_ids: vec![s()],
		repos: v()
	})
	.is_ok());
	assert!(check_empty_profile(&Profile {
		name: s(),
		output_dir: p(),
		game_version: s(),
		mod_loader: s(),
		mod_ids: v(),
		repos: vec![(s(), s())]
	})
	.is_ok());
	assert!(check_empty_profile(&Profile {
		name: s(),
		output_dir: p(),
		game_version: s(),
		mod_loader: s(),
		mod_ids: vec![s()],
		repos: vec![(s(), s())]
	})
	.is_ok());
	assert!(check_empty_profile(&Profile {
		name: s(),
		output_dir: p(),
		game_version: s(),
		mod_loader: s(),
		mod_ids: v(),
		repos: v()
	})
	.is_err());
}

#[tokio::test]
async fn test_add_mod_modrinth() {
	let modrinth = ferinth::Ferinth::new("ferium-test");

	let mut config = Profile {
		name: s(),
		output_dir: p(),
		game_version: s(),
		mod_loader: s(),
		mod_ids: v(),
		repos: v(),
	};

	assert!(add_mod_modrinth(&modrinth, "aanobbmi".into(), &mut config)
		.await
		.is_err());

	add_mod_modrinth(&modrinth, "sodium".into(), &mut config)
		.await
		.unwrap();
	assert!(config.mod_ids.contains(&"AANobbMI".into()));

	assert!(add_mod_modrinth(&modrinth, "sodium".into(), &mut config)
		.await
		.is_err())
}

#[tokio::test]
async fn test_add_repo_github() {
	let github = octocrab::instance();

	let mut config = Profile {
		name: s(),
		output_dir: p(),
		game_version: s(),
		mod_loader: s(),
		mod_ids: v(),
		repos: v(),
	};

	assert!(
		add_repo_github(&github, "p".into(), "p".into(), &mut config)
			.await
			.is_err()
	);

	add_repo_github(
		&github,
		"caffeinemc".into(),
		"Sodium-Fabric".into(),
		&mut config,
	)
	.await
	.unwrap();
	assert!(config
		.repos
		.contains(&("CaffeineMC".into(), "sodium-fabric".into())));
}

#[tokio::test]
async fn test_get_latest_mc_versions() -> FResult<()> {
	let versions = get_latest_mc_versions(6).await?;
	println!("{:#?}", versions);
	assert!(versions.len() == 6);

	Ok(())
}

#[test]
fn test_remove_semver_patch() -> FResult<()> {
	assert_eq!(remove_semver_patch("1.7.10")?, "1.7".to_string());
	assert_eq!(remove_semver_patch("1.14.4")?, "1.14".to_string());
	assert_eq!(remove_semver_patch("1.14")?, "1.14".to_string());

	Ok(())
}
