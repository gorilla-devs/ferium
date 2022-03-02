use libium::config;

pub fn list(config: &config::structs::Config) {
	for profile in &config.profiles {
		println!(
			"{}
		\r  Output directory:    {:?}
		\r  Minecraft Version:   {}
		\r  Mod Loader:          {}
		\r  CurseForge Projects: {}
		\r  Modrinth Mods:       {}
		\r  GitHub Repositories: {}\n",
			profile.name,
			profile.output_dir,
			profile.game_version,
			profile.mod_loader,
			profile.curse_projects.len(),
			profile.modrinth_mods.len(),
			profile.github_repos.len(),
		);
	}
}
