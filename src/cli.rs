#![deny(missing_docs)]

use crate::DEFAULT_PARALLEL_NETWORK;
use clap::{Args, Parser, Subcommand, ValueEnum, ValueHint};
use clap_complete::Shell;
use libium::config::{
    filters::{self, Filter},
    structs::ModLoader,
};
use std::path::PathBuf;

#[derive(Clone, Debug, Parser)]
#[clap(author, version, about)]
#[clap(arg_required_else_help = true)]
pub struct Ferium {
    #[clap(subcommand)]
    pub subcommand: SubCommands,
    /// Sets the number of worker threads the tokio runtime will use.
    /// You can also use the environment variable `TOKIO_WORKER_THREADS`.
    #[clap(long, short)]
    pub threads: Option<usize>,
    /// Specify the maximum number of parallel network requests to perform.
    #[clap(long, short = 'p', default_value_t = DEFAULT_PARALLEL_NETWORK)]
    pub parallel_network: usize,
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
    /// Whether no gui mode is enabled or not.
    /// You can also use the environment variable `FERIUM_NO_GUI`.
    #[clap(short, long, visible_alias = "ng")]
    pub no_gui: Option<bool>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum SubCommands {
    /*  TODO:
        Use this for filter arguments:
        https://docs.rs/clap/latest/clap/_derive/_tutorial/chapter_3/index.html#argument-relations
    */
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

        #[command(flatten)]
        filters: FilterArguments,
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

#[derive(Clone, Debug, Subcommand)]
pub enum ProfileSubCommands {
    /// Configure the current profile's name, Minecraft version, mod loader, and output directory.
    /// Optionally, provide the settings to change as arguments.
    #[clap(visible_aliases = ["config", "conf"])]
    Configure {
        /// The Minecraft version(s) to consider as compatible
        #[clap(long, short = 'v')]
        game_versions: Vec<String>,
        /// The mod loader(s) to consider as compatible
        #[clap(long, short = 'l')]
        #[clap(value_enum)]
        mod_loaders: Vec<ModLoader>,
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
        #[expect(clippy::option_option)]
        import: Option<Option<String>>,
        /// The Minecraft version to check compatibility for
        #[clap(long, short = 'v')]
        game_version: Vec<String>,
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
    /// Import an existing profile
    Import {
        /// The name of the profile
        #[clap(long, short)]
        name: Option<String>,
        /// The path to the profile
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::FilePath))]
        path: Option<PathBuf>,
        /// The directory the profile will output mods to
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        output_dir: Option<PathBuf>,
    },
    /// Switch between different profiles.
    /// Optionally, provide the name of the profile to switch to.
    Switch {
        /// The name of the profile to switch to
        profile_name: Option<String>,
    },
}

#[derive(Clone, Debug, Subcommand)]
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

#[derive(Clone, Default, Debug, Args)]
#[group(id = "loader", multiple = false)]
pub struct FilterArguments {
    #[clap(long, short = 'p')]
    pub override_profile: bool,

    #[clap(long, short = 'l', group = "loader")]
    pub mod_loader_prefer: Vec<ModLoader>,
    #[clap(long, group = "loader")]
    pub mod_loader_any: Vec<ModLoader>,

    #[clap(long, short = 'v', group = "version")]
    pub game_version_strict: Vec<String>,
    #[clap(long, group = "version")]
    pub game_version_minor: Vec<String>,

    #[clap(long, short = 'c')]
    pub release_channel: Option<filters::ReleaseChannel>,

    #[clap(long, short = 'n')]
    pub filename: Option<String>,
    #[clap(long, short = 't')]
    pub title: Option<String>,
    #[clap(long, short = 'd')]
    pub description: Option<String>,
}

impl From<FilterArguments> for Vec<Filter> {
    fn from(value: FilterArguments) -> Self {
        let mut filters = vec![];

        if !value.mod_loader_prefer.is_empty() {
            filters.push(Filter::ModLoaderPrefer(value.mod_loader_prefer));
        }
        if !value.mod_loader_any.is_empty() {
            filters.push(Filter::ModLoaderAny(value.mod_loader_any));
        }
        if !value.game_version_strict.is_empty() {
            filters.push(Filter::GameVersionStrict(value.game_version_strict));
        }
        if !value.game_version_minor.is_empty() {
            filters.push(Filter::GameVersionMinor(value.game_version_minor));
        }
        if let Some(release_channel) = value.release_channel {
            filters.push(Filter::ReleaseChannel(release_channel));
        }
        if let Some(regex) = value.filename {
            filters.push(Filter::Filename(regex));
        }
        if let Some(regex) = value.title {
            filters.push(Filter::Title(regex));
        }
        if let Some(regex) = value.description {
            filters.push(Filter::Description(regex));
        }

        filters
    }
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
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
