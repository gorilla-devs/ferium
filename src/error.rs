//! Contains error handling helpers

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Your currently selected profile is empty! Run `ferium help` to see how to add mods")]
	EmptyConfigFile,
	#[error("{}", .0)]
	ReqwestError(#[from] reqwest::Error),
	#[error("{}", .0)]
	OctocrabError(octocrab::Error),
	#[error(
		"JSON {} error at {}, {}",
		match .0.classify() {
			serde_json::error::Category::Syntax => {
				"syntax"
			},
			serde_json::error::Category::Data => {
				"non matching type"
			},
			serde_json::error::Category::Eof => {
				"unexpected end of file"
			},
			serde_json::error::Category::Io => {
				"input/output"
			},
    	},
		.0.line(),
		.0.column()
	)]
	JSONError(#[from] serde_json::error::Error),
	#[error("{}", .0)]
	IOError(#[from] std::io::Error),
	#[error("{}", .0)]
	FerinthError(ferinth::Error),
	#[error("{}", .0)]
	SemverError(#[from] semver::Error),
	#[error("{}", .0)]
	URLParseError(url::ParseError),
	#[error("{}", .0)]
	AddError(libium::add::Error),
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
			octocrab::Error::Http { source, .. } => Self::ReqwestError(source),
			octocrab::Error::Json { source, .. } => Self::JSONError(source.into_inner()),
			octocrab::Error::Serde { source, .. } => Self::JSONError(source),
			octocrab::Error::Url { source, .. } => Self::URLParseError(source),
			_ => Self::OctocrabError(err),
		}
	}
}

impl From<ferinth::Error> for Error {
	fn from(err: ferinth::Error) -> Self {
		match err {
			ferinth::Error::ReqwestError(err) => Self::ReqwestError(err),
			ferinth::Error::URLParseError(err) => Self::URLParseError(err),
			_ => Self::FerinthError(err),
		}
	}
}

impl From<libium::add::Error> for Error {
	fn from(err: libium::add::Error) -> Self {
		match err {
			libium::add::Error::GitHubError(err) => err.into(),
			libium::add::Error::ModrinthError(err) => err.into(),
			libium::add::Error::CurseForgeError(err) => err.into(),
			_ => Self::AddError(err),
		}
	}
}
