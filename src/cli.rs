use clap::{Parser, Subcommand, ValueHint};
use clap_complete::Shell;
use libium::config::structs::ModLoader;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(propagate_version = true)]
#[clap(arg_required_else_help = true)]
pub struct Ferium {
    #[clap(subcommand)]
    pub subcommand: SubCommands,
    #[clap(long, short)]
    #[clap(help("The limit for additional threads spawned by the Tokio runtime"))]
    pub threads: Option<usize>,
    #[clap(long)]
    #[clap(help("A GitHub personal access token for increasing the GitHub API rate limit"))]
    pub github_token: Option<String>,
    #[clap(long)]
    #[clap(help(
        "Set the file to read the config from. Does not change the cache and tmp directories"
    ))]
    #[clap(value_hint(ValueHint::FilePath))]
    pub config_file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum SubCommands {
    #[clap(about("Add a mod to the profile"))]
    Add {
        #[clap(help("The identifier of the mod/project/repository
The Modrinth project ID is specified at the bottom of the left sidebar under 'Technical information'. You can also use the project slug in the URL
The CurseForge mod ID is specified at the top of the right sidebar under 'About Project'
The GitHub identifier is the repository's full name, e.g. `gorilla-devs/ferium`"))]
        identifier: String,
        #[clap(long)]
        #[clap(help("The game version will not be checked for this mod"))]
        dont_check_game_version: bool,
        #[clap(long)]
        #[clap(help("The mod loader will not be checked for this mod"))]
        dont_check_mod_loader: bool,
        #[clap(long)]
        #[clap(help("The mod's dependencies will not be added"))]
        dont_add_dependencies: bool,
    },
    #[clap(about("Generate shell auto completions to stdout for the specified shell"))]
    Complete {
        #[clap(help("The shell to generate auto completions for"))]
        shell: Shell,
    },
    #[clap(about("List all the mods in the profile, and with some their metadata if verbose"))]
    List {
        #[clap(long, short)]
        #[clap(help("Show information about the mod"))]
        verbose: bool,
        #[clap(long)]
        #[clap(help(
            "Output information in markdown format and alphabetical order, e.g. for modpack mod lists
Complements the verbose flag"
        ))]
        markdown: bool,
    },
    #[clap(arg_required_else_help = true)]
    #[clap(about("Add, configure, delete, switch, list, or upgrade modpacks"))]
    Modpack {
        #[clap(subcommand)]
        subcommand: ModpackSubCommands,
    },
    #[clap(arg_required_else_help = true)]
    #[clap(about("Create, configure, delete, switch, or list profiles"))]
    Profile {
        #[clap(subcommand)]
        subcommand: ProfileSubCommands,
    },
    #[clap(about(
        "Remove a mod or repository from the profile
Optionally, provide a list of names of the mods to remove"
    ))]
    Remove {
        #[clap(name("mod-name"))]
        #[clap(help("A case-insensitive list of names of a mods to remove"))]
        mod_names: Vec<String>,
    },
    #[clap(about("Download and install the latest version of the mods specified"))]
    Upgrade,
    #[clap(about("Scan profile for mods"))]
    Scan {
        #[clap(long)]
        #[clap(arg_enum)]
        #[clap(help("Preferred platform to scan mods from (default: modrinth)"))]
        preferred_platform: Option<libium::config::structs::ModPlatform>
    }
}

#[derive(Subcommand)]
pub enum ProfileSubCommands {
    #[clap(about(
        "Configure the current profile's Minecraft version, mod loader, and output directory
Optionally, provide setting(s) to change as option(s)"
    ))]
    Configure {
        #[clap(long)]
        #[clap(help("The Minecraft version to check compatibility for"))]
        game_version: Option<String>,
        #[clap(long)]
        #[clap(arg_enum)]
        #[clap(help("The mod loader to check compatibility for"))]
        mod_loader: Option<ModLoader>,
        #[clap(long)]
        #[clap(help("The name of the profile"))]
        name: Option<String>,
        #[clap(long)]
        #[clap(value_hint(ValueHint::DirPath))]
        #[clap(help("The directory to output mods to"))]
        output_dir: Option<PathBuf>,
    },
    #[clap(about(
        "Create a new profile
Optionally provide all the options, to create the profile without the UI
Use the import flag to import mods from another profile"
    ))]
    Create {
        #[clap(long)]
        #[allow(clippy::option_option)]
        #[clap(help(
            "Copy over the mods from an existing profile
Optionally, provide the name of the profile to import mods from"
        ))]
        import: Option<Option<String>>,
        #[clap(long)]
        #[clap(help("The Minecraft version to check compatibility for"))]
        game_version: Option<String>,
        #[clap(long)]
        #[clap(arg_enum)]
        #[clap(help("The mod loader to check compatibility for"))]
        mod_loader: Option<ModLoader>,
        #[clap(long)]
        #[clap(help("The name of the profile"))]
        name: Option<String>,
        #[clap(long)]
        #[clap(value_hint(ValueHint::DirPath))]
        #[clap(help("The directory to output mods to"))]
        output_dir: Option<PathBuf>,
    },
    #[clap(about(
        "Delete a profile
Optionally, provide the name of the profile to delete"
    ))]
    Delete {
        #[clap(long)]
        #[clap(help("The name of the profile to delete"))]
        profile_name: Option<String>,
    },
    #[clap(about("List all the profiles with their data"))]
    List,
    #[clap(about(
        "Switch between different profiles
Optionally, provide the name of the profile to switch to"
    ))]
    Switch {
        #[clap(long)]
        #[clap(help("The name of the profile to switch to"))]
        profile_name: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ModpackSubCommands {
    #[clap(about("Add a modpack to the config"))]
    Add {
        #[clap(help("The identifier of the modpack/project
The Modrinth project ID is specified at the bottom of the left sidebar under 'Technical information'. You can also use the project slug for this
The CurseForge mod ID is specified at the top of the right sidebar under 'About Project'"))]
        identifier: String,
        #[clap(long)]
        #[clap(value_hint(ValueHint::DirPath))]
        #[clap(help("The Minecraft instance directory to install the modpack to"))]
        output_dir: Option<PathBuf>,
        #[clap(long)]
        #[clap(help(
            "Whether to install the modpack's overrides to the output directory
This will override existing files"
        ))]
        install_overrides: Option<bool>,
    },
    #[clap(about(
        "Configure the current modpack's output directory
Optionally, provide the output directory as an option"
    ))]
    Configure {
        #[clap(long)]
        #[clap(value_hint(ValueHint::DirPath))]
        #[clap(help("The Minecraft instance directory to install the modpack to"))]
        output_dir: Option<PathBuf>,
        #[clap(long)]
        #[clap(help(
            "Whether to install the modpack's overrides to the output directory
This may overwrite existing files"
        ))]
        install_overrides: Option<bool>,
    },
    #[clap(about(
        "Delete a modpack
Optionally, provide the name of the modpack to delete"
    ))]
    Delete {
        #[clap(long)]
        #[clap(help("The name of the modpack to delete"))]
        modpack_name: Option<String>,
    },
    #[clap(about("List all the modpacks"))]
    List,
    #[clap(about(
        "Switch between different modpacks
Optionally, provide the name of the modpack to switch to"
    ))]
    Switch {
        #[clap(long)]
        #[clap(help("The name of the modpack to switch to"))]
        modpack_name: Option<String>,
    },
    #[clap(about("Download and install the latest version of the modpack"))]
    Upgrade,
}
