// This file contains miscellanous convenience functions

use shellexpand::tilde;
use std::env::consts::OS;
use std::io::{stdout, Write};
use std::process::exit;

/// Returns the default directory where mods are stored
pub fn get_mods_dir() -> std::borrow::Cow<'static, str> {
    if OS == "macos" {
        tilde("~/Library/ApplicationSupport/minecraft/mods/")
    } else if OS == "linux" {
        tilde("~/.minecraft/mods/")
    } else if OS == "windows" {
        tilde("~\\AppData\\Roaming\\.minecraft\\mods\\")
    } else {
        println!("Not running on a device capable of running Minecraft Java Edition!");
        exit(126)
    }
}

/// Run `print` macro and flush stdout to make results immediately appear
pub fn print(msg: &str) {
    print!("{}", msg);
    stdout().flush().unwrap();
}
