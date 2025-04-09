use std::{
    io::Result,
    path::{Path, PathBuf},
};

use libium::BASE_DIRS;

#[cfg(feature = "gui")]
/// Uses the system file picker to pick a file, with a `default` path
fn show_folder_picker(default: impl AsRef<Path>, prompt: impl Into<String>) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_can_create_directories(true)
        .set_directory(default)
        .set_title(prompt)
        .pick_folder()
}

#[cfg(not(feature = "gui"))]
/// Uses a terminal input to pick a file, with a `default` path
fn show_folder_picker(default: impl AsRef<Path>, prompt: impl Into<String>) -> Option<PathBuf> {
    inquire::Text::new(&prompt.into())
        .with_default(&default.as_ref().display().to_string())
        .prompt()
        .ok()
        .map(Into::into)
}

/// Picks a folder using the terminal or system file picker (depending on the feature flag `gui`)
///
/// The `default` path is shown/opened at first and the `name` is what folder the user is supposed to be picking (e.g. output directory)
pub fn pick_folder(
    default: impl AsRef<Path>,
    prompt: impl Into<String>,
    name: impl AsRef<str>,
) -> Result<Option<PathBuf>> {
    show_folder_picker(default, prompt)
        .map(|raw_in| {
            let path = raw_in
                .components()
                .map(|c| {
                    if c.as_os_str() == "~" {
                        BASE_DIRS.home_dir().as_os_str()
                    } else {
                        c.as_os_str()
                    }
                })
                .collect::<PathBuf>()
                .canonicalize()?;

            println!(
                "✔ \x1b[01m{}\x1b[0m · \x1b[32m{}\x1b[0m",
                name.as_ref(),
                path.display(),
            );

            Ok(path)
        })
        .transpose()
}
