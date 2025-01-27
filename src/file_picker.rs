use colored::Colorize;
use libium::HOME;
use std::{
    env::VarError, io, path::{Path, PathBuf}
};

/// Picks a folder using the terminal or system file picker (depending on the feature flag `gui`)
///
/// The `default` path is shown/opened at first and the `name` is what folder the user is supposed to be picking (e.g. output directory)
pub fn pick_folder(
    default: impl AsRef<Path>,
    prompt: impl Into<String>,
    name: impl AsRef<str>,
    no_gui: Option<bool>,
) -> io::Result<Option<PathBuf>> {
    // Check if no_gui is enabled or disabled
    // If the no_gui parameter is none then try parse the environment variable $FERIUM_NO_GUI 
    let no_gui = no_gui.unwrap_or_else(|| {
        match std::env::var("FERIUM_NO_GUI") {
            Ok(x) => match x.parse() {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("Failed to parse environment variable `FERIUM_NO_GUI`: {e}");
                    true
                },
            },
            Err(e) => match e {
                VarError::NotPresent => false,
                VarError::NotUnicode(string) => {
                    eprintln!("environment variable `FERIUM_NO_GUI` has invalid unicode characters: {string:?}");
                    true
                },
            },
        }
    });

    // Ask the user for the folder location
    // using rfd if gui is enabled and the terminal if it is not
    let path = if no_gui {
        inquire::Text::new(&prompt.into())
            .with_default(&default.as_ref().display().to_string())
            .prompt()
            .ok()
            .map(|path| PathBuf::from(path))
    } else {
        rfd::FileDialog::new()
            .set_can_create_directories(true)
            .set_directory(default)
            .set_title(prompt)
            .pick_folder()
    };
 
    match path {
        Some(path) => {
            // '~' is an alias for home directory on Unix-based OS
            // so replace '~' with path to home
            let path: PathBuf = path.components()
            .map(|c| if c.as_os_str() == "~" {
                HOME.as_os_str()
            } else {
                c.as_os_str()
            })
            .collect();

            // Convert to absolute path
            let path = path.canonicalize()?;
            
            println!(
                "✔ {} · {}",
                name.as_ref().bold(),
                path.display().to_string().green(),
            );

            Ok(Some(path))
        },
        None => Ok(None),
    }
}

#[cfg(feature = "gui")]
/// Uses the system file picker to pick a file, with a `default` path
fn show_file_picker(default: impl AsRef<Path>, prompt: impl Into<String>) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_can_create_directories(true)
        .set_directory(default)
        .set_title(prompt)
        .pick_file()
}

#[cfg(not(feature = "gui"))]
/// Uses a terminal input to pick a file, with a `default` path
fn show_file_picker(default: impl AsRef<Path>, prompt: impl Into<String>) -> Option<PathBuf> {
    inquire::Text::new(&prompt.into())
        .with_default(&default.as_ref().display().to_string())
        .prompt()
        .ok()
        .map(Into::into)
}

/// Picks a folder using the terminal or system file picker (depending on the feature flag `gui`)
///
/// The `default` path is shown/opened at first and the `name` is what folder the user is supposed to be picking (e.g. output directory)
pub fn pick_file(
    default: impl AsRef<Path>,
    prompt: impl Into<String>,
    name: impl AsRef<str>,
) -> Result<Option<PathBuf>> {
    show_file_picker(default, prompt)
        .map(|raw_in| {
            let path = raw_in
                .components()
                .map(|c| {
                    if c.as_os_str() == "~" {
                        HOME.as_os_str()
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
