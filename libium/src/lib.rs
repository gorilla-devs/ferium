pub mod add;
pub mod config;
pub mod iter_ext;
pub mod modpack;
pub mod scan;
pub mod upgrade;
pub mod version_ext;

pub use add::add;
pub use scan::scan;

use directories::{BaseDirs, ProjectDirs};
use std::{path::PathBuf, sync::LazyLock};

pub static GITHUB_API: LazyLock<octocrab::Octocrab> = LazyLock::new(|| {
    let mut github = octocrab::OctocrabBuilder::new();
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        github = github.personal_token(token);
    }
    github.build().expect("Could not build GitHub client")
});

pub static CURSEFORGE_API: LazyLock<furse::Furse> = LazyLock::new(|| {
    furse::Furse::new(std::env::var("CURSEFORGE_API_KEY").unwrap_or(String::from(
        "$2a$10$sI.yRk4h4R49XYF94IIijOrO4i3W3dAFZ4ssOlNE10GYrDhc2j8K.",
    )))
});

pub static MODRINTH_API: LazyLock<ferinth::Ferinth<()>> = LazyLock::new(|| {
    ferinth::Ferinth::<()>::new(
        "ferium",
        // TODO: option_env!("CARGO_PKG_VERSION"),
        None,
        Some("Discord: therookiecoder"),
    )
});

pub static BASE_DIRS: LazyLock<BaseDirs> =
    LazyLock::new(|| BaseDirs::new().expect("Could not get OS specific directories"));

pub static PROJECT_DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("", "", "ferium").expect("Could not get OS specific directories")
});

/// Gets the default Minecraft instance directory based on the current compilation `target_os`
pub fn get_minecraft_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        BASE_DIRS.data_dir().join("minecraft")
    }
    #[cfg(target_os = "windows")]
    {
        BASE_DIRS.data_dir().join(".minecraft")
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        BASE_DIRS.home_dir().join(".minecraft")
    }
}

/// Read `source` and return the data as a string
///
/// A wrapper for dealing with the read buffer.
pub fn read_wrapper(mut source: impl std::io::Read) -> std::io::Result<String> {
    let mut buffer = String::new();
    source.read_to_string(&mut buffer)?;
    Ok(buffer)
}
