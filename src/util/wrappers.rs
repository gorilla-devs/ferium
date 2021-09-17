// This file contains miscellanous convenience functions

use shellexpand::tilde;
use std::env::consts::OS;
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};

/// Returns the default directory where mods are stored
pub fn get_mods_dir() -> String {
    let home = tilde("~");
    let home = Path::new(home.as_ref());
    let path: PathBuf;

    if OS == "macos" {
        path = home
            .join("Library")
            .join("ApplicationSupport")
            .join("minecraft")
            .join("mods");
    } else if OS == "linux" {
        path = home.join(".minecraft").join("mods");
    } else if OS == "windows" {
        tilde("~\\AppData\\Roaming\\.minecraft\\mods\\");
        path = home
            .join("AppData")
            .join("Roaming")
            .join(".minecraft")
            .join("mods");
    } else {
        panic!("Not running on a device capable of running Minecraft Java Edition!");
    }

    path.to_str().unwrap().into()
}

/// Run `print` macro and flush stdout to make results immediately appear
pub fn print(msg: impl std::fmt::Display) {
    print!("{}", msg);
    stdout().flush().unwrap();
}
