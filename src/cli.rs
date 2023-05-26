#![deny(missing_docs)]

use clap::{clap_derive::ValueEnum, Parser, Subcommand, ValueHint};
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
    /// Sets the number of worker threads the tokio runtime will use.
    /// You can also use the environment variable `TOKIO_WORKER_THREADS`.
    pub threads: Option<usize>,
    #[clap(long)]
    /// Set a GitHub personal access token for increasing the GitHub API rate limit.
    /// You can also use the environment variable `GITHUB_TOKEN`.
    pub github_token: Option<String>,
    #[clap(long)]
    /// Set a custom CurseForge API key.
    /// You can also use the environment variable `CURSEFORGE_API_KEY`.
    pub curseforge_api_key: Option<String>,
    #[clap(long, short)]
    #[clap(value_hint(ValueHint::FilePath))]
    /// Set the file to read the config from.
    /// This does not change the `cache` and `tmp` directories.
    /// You can also use the environment variable `CONFIG_FILE`.
    pub config_file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum SubCommands {
    /// Add a mod to the profile
    Add {
        /// The identifier of the mod/project/repository
        ///
        /// The Modrinth project ID is specified at the bottom of the left sidebar under 'Technical information'.
        /// You can also use the project slug in the URL.
        /// The CurseForge project ID is specified at the top of the right sidebar under 'About Project'.
        /// The GitHub identifier is the repository's full name, e.g. `gorilla-devs/ferium`.
        identifier: String,
        #[clap(long)]
        /// The game version will not be checked for this mod
        dont_check_game_version: bool,
        #[clap(long)]
        /// The mod loader will not be checked for this mod
        dont_check_mod_loader: bool,
        #[clap(long)]
        #[clap(value_enum)]
        /// Select which dependencies should be added
        dependencies: Option<DependencyLevel>,
    },
    /// Print shell auto completions for the specified shell
    Complete {
        #[clap(value_enum)]
        /// The shell to generate auto completions for
        shell: Shell,
    },
    /// List all the mods in the profile, and with some their metadata if verbose
    List {
        #[clap(long, short)]
        /// Show additional information about the mod
        verbose: bool,
        #[clap(long, short)]
        /// Like verbose, but outputs information in markdown format and ordered alphabetically
        /// 
        /// Useful for creating modpack mod lists.
        markdown: bool,
    },
    #[clap(arg_required_else_help = true)]
    /// Add, configure, delete, switch, list, or upgrade modpacks
    Modpack {
        #[clap(subcommand)]
        subcommand: ModpackSubCommands,
    },
    #[clap(arg_required_else_help = true)]
    /// Create, configure, delete, switch, or list profiles
    Profile {
        #[clap(subcommand)]
        subcommand: ProfileSubCommands,
    },
    /// Remove mods and repositories from the profile.
    /// Optionally, provide a list of names or IDs of the mods to remove.
    Remove {
        /// List of project IDs or case-insensitive names of mods to remove
        mod_names: Vec<String>,
    },
    /// Download and install the latest compatible version of your mods
    Upgrade,
}

#[derive(Subcommand)]
pub enum ProfileSubCommands {
    /// Configure the current profile's name, Minecraft version, mod loader, and output directory.
    /// Optionally, provide the settings to change as arguments.
    Configure {
        #[clap(long, short = 'v')]
        /// The Minecraft version to check compatibility for
        game_version: Option<String>,
        #[clap(long, short)]
        #[clap(value_enum)]
        /// The mod loader to check compatibility for
        mod_loader: Option<ModLoader>,
        #[clap(long, short)]
        /// The name of the profile
        name: Option<String>,
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        /// The directory to output mods to
        output_dir: Option<PathBuf>,
    },
    /// Create a new profile.
    /// Optionally, provide the settings as arguments.
    /// Use the import flag to import mods from another profile.
    Create {
        #[clap(long, short)]
        #[allow(clippy::option_option)]
        /// Copy over the mods from an existing profile.
        /// Optionally, provide the name of the profile to import mods from.
        import: Option<Option<String>>,
        #[clap(long, short = 'v')]
        /// The Minecraft version to check compatibility for
        game_version: Option<String>,
        #[clap(long, short)]
        #[clap(value_enum)]
        /// The mod loader to check compatibility for
        mod_loader: Option<ModLoader>,
        #[clap(long, short)]
        /// The name of the profile
        name: Option<String>,
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        /// The directory to output mods to
        output_dir: Option<PathBuf>,
    },
    /// Delete a profile.
    /// Optionally, provide the name of the profile to delete.
    Delete {
        /// The name of the profile to delete
        profile_name: Option<String>,
    },
    /// Show active profile's information
    Info,
    /// List all the profiles with their data
    List,
    /// Switch between different profiles.
    /// Optionally, provide the name of the profile to switch to.
    Switch {
        /// The name of the profile to switch to
        profile_name: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ModpackSubCommands {
    /// Add a modpack to the config
    Add {
        /// The identifier of the modpack/project
        ///
        /// The Modrinth project ID is specified at the bottom of the left sidebar under 'Technical information'.
        /// You can also use the project slug for this.
        /// The CurseForge project ID is specified at the top of the right sidebar under 'About Project'.
        identifier: String,
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        /// The Minecraft instance directory to install the modpack to
        output_dir: Option<PathBuf>,
        #[clap(long, short)]
        /// Whether to install the modpack's overrides to the output directory.
        /// This will override existing files when upgrading.
        install_overrides: Option<bool>,
    },
    /// Configure the current modpack's output directory and installation of overrides.
    /// Optionally, provide the settings to change as arguments.
    Configure {
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        /// The Minecraft instance directory to install the modpack to
        output_dir: Option<PathBuf>,
        #[clap(long, short)]
        /// Whether to install the modpack's overrides to the output directory.
        /// This will override existing files when upgrading.
        install_overrides: Option<bool>,
    },
    /// Delete a modpack.
    /// Optionally, provide the name of the modpack to delete.
    Delete {
        /// The name of the modpack to delete
        modpack_name: Option<String>,
    },
    /// List all the modpacks
    List,
    /// Switch between different modpacks.
    /// Optionally, provide the name of the modpack to switch to.
    Switch {
        /// The name of the modpack to switch to
        modpack_name: Option<String>,
    },
    /// Download and install the latest version of the modpack
    Upgrade,
}

#[derive(Clone, PartialEq, Eq, ValueEnum)]
pub enum DependencyLevel {
    /// Do not add any dependencies
    None,
    /// Add only required dependencies
    Required,
    /// Add all dependencies
    All,
}
