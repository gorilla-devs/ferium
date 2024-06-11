#![deny(missing_docs)]

use clap::{Parser, Subcommand, ValueEnum, ValueHint};
use clap_complete::Shell;
use libium::config::structs::ModLoader;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(arg_required_else_help = true)]
pub struct Ferium {
    #[clap(subcommand)]
    pub subcommand: SubCommands,
    /// Sets the number of worker threads the tokio runtime will use.
    /// You can also use the environment variable `TOKIO_WORKER_THREADS`.
    #[clap(long, short)]
    pub threads: Option<usize>,
    /// Set a GitHub personal access token for increasing the GitHub API rate limit.
    /// You can also use the environment variable `GITHUB_TOKEN`.
    #[clap(long, visible_alias = "gh")]
    pub github_token: Option<String>,
    /// Set a custom Curseforge API key.
    /// You can also use the environment variable `CURSEFORGE_API_KEY`.
    #[clap(long, visible_alias = "cf")]
    pub curseforge_api_key: Option<String>,
    /// Set the file to read the config from.
    /// This does not change the `cache` and `tmp` directories.
    /// You can also use the environment variable `FERIUM_CONFIG_FILE`.
    #[clap(long, short, visible_aliases = ["config", "conf"])]
    #[clap(value_hint(ValueHint::FilePath))]
    pub config_file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum SubCommands {
    /// Add mods to the profile
    Add {
        /// The identifier(s) of the mod/project/repository
        ///
        /// The Modrinth project ID is specified at the bottom of the left sidebar under 'Technical information'.
        /// You can also use the project slug in the URL.
        /// The Curseforge project ID is specified at the top of the right sidebar under 'About Project'.
        /// The GitHub identifier is the repository's full name, e.g. `gorilla-devs/ferium`.
        #[clap(required = true)]
        identifiers: Vec<String>,
        /// Temporarily ignore game version and mod loader checks and add the mod anyway
        #[clap(long, short, visible_alias = "override")]
        force: bool,
        /// The game version will not be checked for this mod.
        /// Only works when adding a single mod.
        #[clap(long, short = 'V', alias = "dont-check-game-version")]
        ignore_game_version: bool,
        /// The mod loader will not be checked for this mod.
        /// Only works when adding a single mod.
        #[clap(long, short = 'M', alias = "dont-check-mod-loader")]
        ignore_mod_loader: bool,
    },
    /// Scan the profile's output directory (or the specified directory) for mods and add them to the profile
    Scan {
        /// The platform you prefer mods to be added from.
        /// If a mod isn't available from this platform, the other platform will still be used.
        #[clap(long, short, default_value_t)]
        platform: Platform,
        /// The directory to scan mods from.
        /// Defaults to the profile's output directory.
        #[clap(long, short,
            visible_aliases = ["dir", "folder"],
            aliases = ["output_directory", "out_dir"]
        )]
        directory: Option<PathBuf>,
        /// Temporarily ignore game version and mod loader checks and add the mods anyway
        #[clap(long, short, visible_alias = "override")]
        force: bool,
    },
    /// Print shell auto completions for the specified shell
    Complete {
        /// The shell to generate auto completions for
        #[clap(value_enum)]
        shell: Shell,
    },
    /// List all the mods in the profile, and with some their metadata if verbose
    #[clap(visible_alias = "mods")]
    List {
        /// Show additional information about the mod
        #[clap(long, short)]
        verbose: bool,
        /// Output information in markdown format and alphabetical order
        ///
        /// Useful for creating modpack mod lists.
        /// Complements the verbose flag.
        #[clap(long, short, visible_alias = "md")]
        markdown: bool,
    },
    /// Add, configure, delete, switch, list, or upgrade modpacks
    Modpack {
        #[clap(subcommand)]
        subcommand: Option<ModpackSubCommands>,
    },
    /// List all the modpacks with their data
    Modpacks,
    /// Create, configure, delete, switch, or list profiles
    Profile {
        #[clap(subcommand)]
        subcommand: Option<ProfileSubCommands>,
    },
    /// List all the profiles with their data
    Profiles,
    /// Remove mods and/or repositories from the profile.
    /// Optionally, provide a list of names or IDs of the mods to remove.
    #[clap(visible_alias = "rm")]
    Remove {
        /// List of project IDs or case-insensitive names of mods to remove
        mod_names: Vec<String>,
    },
    /// Download and install the latest compatible version of your mods
    #[clap(visible_aliases = ["download", "install"])]
    Upgrade,
}

#[derive(Subcommand)]
pub enum ProfileSubCommands {
    /// Configure the current profile's name, Minecraft version, mod loader, and output directory.
    /// Optionally, provide the settings to change as arguments.
    #[clap(visible_aliases = ["config", "conf"])]
    Configure {
        /// The Minecraft version to check compatibility for
        #[clap(long, short = 'v')]
        game_version: Option<String>,
        /// The mod loader to check compatibility for
        #[clap(long, short)]
        #[clap(value_enum)]
        mod_loader: Option<ModLoader>,
        /// The name of the profile
        #[clap(long, short)]
        name: Option<String>,
        /// The directory to output mods to
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        output_dir: Option<PathBuf>,
    },
    /// Create a new profile.
    /// Optionally, provide the settings as arguments.
    /// Use the import flag to import mods from another profile.
    #[clap(visible_alias = "new")]
    Create {
        /// Copy over the mods from an existing profile.
        /// Optionally, provide the name of the profile to import mods from.
        #[clap(long, short, visible_aliases = ["copy", "duplicate"])]
        #[allow(clippy::option_option)]
        import: Option<Option<String>>,
        /// The Minecraft version to check compatibility for
        #[clap(long, short = 'v')]
        game_version: Option<String>,
        /// The mod loader to check compatibility for
        #[clap(long, short)]
        #[clap(value_enum)]
        mod_loader: Option<ModLoader>,
        /// The name of the profile
        #[clap(long, short)]
        name: Option<String>,
        /// The directory to output mods to
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        output_dir: Option<PathBuf>,
    },
    /// Delete a profile.
    /// Optionally, provide the name of the profile to delete.
    #[clap(visible_aliases = ["remove", "rm"])]
    Delete {
        /// The name of the profile to delete
        profile_name: Option<String>,
        /// The name of the profile to switch to afterwards
        #[clap(long, short)]
        switch_to: Option<String>,
    },
    /// Show information about the current profile
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
        /// The Curseforge project ID is specified at the top of the right sidebar under 'About Project'.
        identifier: String,
        /// The Minecraft instance directory to install the modpack to
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        output_dir: Option<PathBuf>,
        /// Whether to install the modpack's overrides to the output directory.
        /// This will override existing files when upgrading.
        #[clap(long, short)]
        install_overrides: Option<bool>,
    },
    /// Configure the current modpack's output directory and installation of overrides.
    /// Optionally, provide the settings to change as arguments.
    #[clap(visible_aliases = ["config", "conf"])]
    Configure {
        /// The Minecraft instance directory to install the modpack to
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        output_dir: Option<PathBuf>,
        /// Whether to install the modpack's overrides to the output directory.
        /// This will override existing files when upgrading.
        #[clap(long, short)]
        install_overrides: Option<bool>,
    },
    /// Delete a modpack.
    /// Optionally, provide the name of the modpack to delete.
    #[clap(visible_aliases = ["remove", "rm"])]
    Delete {
        /// The name of the modpack to delete
        modpack_name: Option<String>,
        /// The name of the profile to switch to afterwards
        #[clap(long, short)]
        switch_to: Option<String>,
    },
    /// Show information about the current modpack
    Info,
    /// List all the modpacks with their data
    List,
    /// Switch between different modpacks.
    /// Optionally, provide the name of the modpack to switch to.
    Switch {
        /// The name of the modpack to switch to
        modpack_name: Option<String>,
    },
    /// Download and install the latest version of the modpack
    #[clap(visible_aliases = ["download", "install"])]
    Upgrade,
}

#[derive(Clone, Copy, Default, ValueEnum)]
pub enum Platform {
    #[default]
    #[clap(alias = "mr")]
    Modrinth,
    #[clap(alias = "cf")]
    Curseforge,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Modrinth => write!(f, "modrinth"),
            Self::Curseforge => write!(f, "curseforge"),
        }
    }
}
