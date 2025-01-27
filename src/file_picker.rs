use colored::Colorize;
use libium::HOME;
use std::{
    env::VarError, io, path::{Path, PathBuf}
};

/// Picks a file using the terminal or system file picker (depending on the `no_gui` parameter, uses the $FERIUM_NO_GUI environment variable if None)
///
/// The `default` path is shown/opened at first and the `name` is what folder/file the user is supposed to be picking (e.g. output directory)
/// 
/// The `is_dir` flag cannot guarantee that the returned path is of the correct kind.
pub fn pick_file(
    default: impl AsRef<Path>,
    prompt: impl Into<String>,
    name: impl AsRef<str>,
    is_dir: bool,
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

    // Ask the user for the folder/file location
    // using rfd if gui is enabled and the terminal if it is not
    let path = if no_gui {
        inquire::Text::new(&prompt.into())
            .with_default(&default.as_ref().display().to_string())
            .prompt()
            .ok()
            .map(|path| PathBuf::from(path))
    } else {
        let fd = rfd::FileDialog::new()
            .set_can_create_directories(true)
            .set_directory(default)
            .set_title(prompt);

        if is_dir {
            fd.pick_folder()
        } else {
            fd.pick_file()
        }
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