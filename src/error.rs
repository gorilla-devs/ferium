//! Contains error handling helpers

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	/// The current profile does not contain mods or repos
	#[error(
		"Your current profile is empty! Run `ferium help` to see how to add mods or repositories"
	)]
	EmptyConfigFile,
	/// An HTTP(S) request returned with an error
	#[error("Failed to send/process an HTTP(S) request due to {}", .0)]
	ReqwestError(#[from] reqwest::Error),
	/// An error encountered in octocrab
	#[error("Error {} occured while using the GitHub API", .0)]
	OctocrabError(String),
	/// Failure to unwrap an Option, akin to `NullPointerError`s
	#[error("Could not access an expected value")]
	OptionError,
	#[error("{}", match .0 {
        serde_json::error::Category::Syntax => {
            "Syntax error encountered in JSON file"
        },
        serde_json::error::Category::Data => {
            "Non matching type while deserialising JSON"
        },
        serde_json::error::Category::Eof => {
            "Unexpected end of file while reading JSON"
        },
        serde_json::error::Category::Io => {
            "Encountered an Input/Output error while handling JSON"
        },
    })]
	JSONError(serde_json::error::Category),
	#[error("Encountered an input/output error, {}", .0.to_string())]
	IOError(#[from] std::io::Error),
	#[error("Invalid request parameter")]
	FerinthBase62Error,
	#[error("Invalid SHA1 hash")]
	FerinthNotSHA1Error,
	#[error("Could not parse a semver version, {}", .0)]
	SemverError(#[from] semver::Error),
	#[error("Could not parse url, {}", .0)]
	URLParseError(url::ParseError),
	/// The application should print `message` and quit (gracefully)
	#[error("{}", .0)]
	Quit(&'static str),
	/// The application should print `message` and quit (gracefully)
	#[error("{}", .0)]
	QuitFormatted(String),
}

impl From<octocrab::Error> for Error {
	fn from(err: octocrab::Error) -> Self {
		match err {
			octocrab::Error::GitHub { source, .. } => Self::OctocrabError(source.message),
			octocrab::Error::Http { source, .. } => Self::ReqwestError(source),
			octocrab::Error::Json { source, .. } => Self::JSONError(source.inner().classify()),
			octocrab::Error::Other { source, .. } => Self::OctocrabError(source.to_string()),
			octocrab::Error::Serde { source, .. } => Self::JSONError(source.classify()),
			octocrab::Error::Url { source, .. } => Self::URLParseError(source),
			octocrab::Error::JWT { source, .. } => Self::OctocrabError(source.to_string()),
		}
	}
}

impl From<ferinth::Error> for Error {
	fn from(err: ferinth::Error) -> Self {
		match err {
			ferinth::Error::NotBase62 => Self::FerinthBase62Error,
			ferinth::Error::NotSHA1 => Self::FerinthNotSHA1Error,
			ferinth::Error::ReqwestError(err) => Self::ReqwestError(err),
    		ferinth::Error::URLParseError(err) => Self::URLParseError(err),
		}
	}
}

impl From<serde_json::error::Error> for Error {
	fn from(err: serde_json::error::Error) -> Self {
		Self::JSONError(err.classify())
	}
}
