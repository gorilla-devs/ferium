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
    /// The limit for additional threads spawned by the Tokio runtime
    pub threads: Option<usize>,
    #[clap(long)]
    /// Set a GitHub personal access token for increasing the GitHub API rate limit.
    /// You can also use the environment variable `GITHUB_TOKEN`
    pub github_token: Option<String>,
    #[clap(long)]
    /// Set a custom CurseForge API key.
    /// You can also use the environment variable `CURSEFORGE_API_KEY`
    pub curseforge_api_key: Option<String>,
    #[clap(long)]
    #[clap(value_hint(ValueHint::FilePath))]
    /// Set the file to read the config from. Does not change the cache and tmp directories
    pub config_file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum SubCommands {
    /// Add a mod to the profile
    Add {
        /// The identifier of the mod/project/repository
        ///
        /// The Modrinth project ID is specified at the bottom of the left sidebar under 'Technical information'. You can also use the project slug in the URL.
        /// The CurseForge mod ID is specified at the top of the right sidebar under 'About Project'.
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
    /// Generate shell auto completions to stdout for the specified shell
    Complete {
        /// The shell to generate auto completions for
        shell: Shell,
    },
    /// List all the mods in the profile, and with some their metadata if verbose
    List {
        #[clap(long, short)]
        /// Show information about the mod
        verbose: bool,
        #[clap(long)]
        /// Output information in markdown format and alphabetical order
        ///
        /// Useful for creating modpack mod lists.
        /// Complements the verbose flag.
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
    /// Remove a mod or repository from the profile
    ///
    /// Optionally, provide a list of names of the mods to remove
    Remove {
        /// A case-insensitive list of names of a mods to remove
        mod_names: Vec<String>,
    },
    /// Download and install the latest version of the mods specified
    Upgrade,
}

#[derive(Subcommand)]
pub enum ProfileSubCommands {
    /// Configure the current profile's Minecraft version, mod loader, and output directory
    ///
    /// Optionally, provide setting(s) to change as option(s)
    Configure {
        #[clap(long)]
        /// The Minecraft version to check compatibility for
        game_version: Option<String>,
        #[clap(long)]
        #[clap(value_enum)]
        /// The mod loader to check compatibility for
        mod_loader: Option<ModLoader>,
        #[clap(long)]
        /// The name of the profile
        name: Option<String>,
        #[clap(long)]
        #[clap(value_hint(ValueHint::DirPath))]
        /// The directory to output mods to
        output_dir: Option<PathBuf>,
    },
    /// Create a new profile
    ///
    /// Optionally provide all the options, to create the profile without the UI.
    /// Use the import flag to import mods from another profile.
    Create {
        #[clap(long)]
        #[allow(clippy::option_option)]
        /// Copy over the mods from an existing profile
        ///
        /// Optionally, provide the name of the profile to import mods from
        import: Option<Option<String>>,
        #[clap(long)]
        /// The Minecraft version to check compatibility for
        game_version: Option<String>,
        #[clap(long)]
        #[clap(value_enum)]
        /// The mod loader to check compatibility for
        mod_loader: Option<ModLoader>,
        #[clap(long)]
        /// The name of the profile
        name: Option<String>,
        #[clap(long)]
        #[clap(value_hint(ValueHint::DirPath))]
        /// The directory to output mods to
        output_dir: Option<PathBuf>,
    },
    /// Delete a profile
    ///
    /// Optionally, provide the name of the profile to delete
    Delete {
        #[clap(long)]
        /// The name of the profile to delete
        profile_name: Option<String>,
    },
    /// Export a profile to a modpack file
    ///
    /// Choose between CurseForge and Modrinth file formats us
    ///
    Export {
        // #[clap(value_enum)]
        // /// The platform to export to
        // platform: Platform,
        #[clap(long)]
        /// The version of the modpack
        modpack_version: Option<String>,
        #[clap(long)]
        /// A short, optional description of this modpack
        summary: Option<String>,
        // #[clap(long)]
        /// The version of the mod loader to load the modpack with
        // mod_loader_version: Option<String>,
        mod_loader_version: String,
        #[clap(long)]
        #[allow(clippy::option_option)]
        #[clap(value_hint(ValueHint::DirPath))]
        /// Whether to include an overrides directory
        /// Optionally, provide the overrides directory from the CLI
        overrides: Option<Option<PathBuf>>,
        #[clap(long)]
        #[allow(clippy::option_option)]
        #[clap(value_hint(ValueHint::DirPath))]
        /// The directory to output the modpack file to
        output_dir: Option<PathBuf>,
    },
    /// List all the profiles with their data
    List,
    /// Switch between different profiles
    ///
    /// Optionally, provide the name of the profile to switch to
    Switch {
        #[clap(long)]
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
        /// The Modrinth project ID is specified at the bottom of the left sidebar under 'Technical information'. You can also use the project slug for this
        /// The CurseForge mod ID is specified at the top of the right sidebar under 'About Project'
        identifier: String,
        #[clap(long)]
        #[clap(value_hint(ValueHint::DirPath))]
        /// The Minecraft instance directory to install the modpack to
        output_dir: Option<PathBuf>,
        #[clap(long)]
        /// Whether to install the modpack's overrides to the output directory
        ///
        /// This will override existing files
        install_overrides: Option<bool>,
    },
    /// Configure the current modpack's output directory
    ///
    /// Optionally, provide the output directory as an option
    Configure {
        #[clap(long)]
        #[clap(value_hint(ValueHint::DirPath))]
        /// The Minecraft instance directory to install the modpack to
        output_dir: Option<PathBuf>,
        #[clap(long)]
        /// Whether to install the modpack's overrides to the output directory
        ///
        /// This may overwrite existing files
        install_overrides: Option<bool>,
    },
    /// Delete a modpack
    ///
    /// Optionally, provide the name of the modpack to delete
    Delete {
        #[clap(long)]
        /// The name of the modpack to delete
        modpack_name: Option<String>,
    },
    /// List all the modpacks
    List,
    /// Switch between different modpacks
    ///
    /// Optionally, provide the name of the modpack to switch to
    Switch {
        #[clap(long)]
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

#[derive(Clone, PartialEq, Eq, ValueEnum)]
pub enum Platform {
    /// The [Modrinth](https://modrinth.com) modding platform
    Modrinth,
    /// The [CurseForge](https://curseforge.com) modding platform
    CurseForge,
}
