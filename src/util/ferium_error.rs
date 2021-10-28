use std::{
    convert::From,
    fmt::{Debug, Formatter},
};

pub type FResult<T> = std::result::Result<T, FError>;

pub enum FError {
    /// The config file does not contain mods or repos
    EmptyConfigFile,
    /// An HTTP(S) request returned with an error
    ReqwestError { error: reqwest::Error },
    /// Failure to unwrap an Option, akin to `NullPointerError`s
    OptionError,
    /// Failed to parse a regular expression
    RegexError,
    /// A JSON error occured
    JsonError {
        category: serde_json::error::Category,
    },
    /// An HTTP(S) request encountered an error
    HTTPError { message: String },
    /// An I/O error occured
    IOError { description: String },
    /// The program is running on an unsupported device
    InvalidDeviceError,
    /// The application should print `message` and quit (gracefully)
    Quit { message: String },
}

impl Debug for FError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            FError::EmptyConfigFile => write!(fmt, "Your config file is empty! Run `ferium help` to see how to add mods or repositories"),
            FError::HTTPError { message } => write!(fmt, "An HTTP(S) request returned an error, {}", message),
            FError::InvalidDeviceError => write!(fmt, "The device you are currently running on is unsupported by Ferium"),
            FError::IOError {description} => write!(fmt, "Encountered an Input/Output error, {}", description),
            FError::JsonError { category } => match category {
                serde_json::error::Category::Syntax => {
                    write!(fmt, "Syntax error encountered in JSON file")
                },
                serde_json::error::Category::Data => {
                    write!(fmt, "Non matching type while deserialising JSON")
                },
                serde_json::error::Category::Eof => {
                    write!(fmt, "Unexpected end of file while reading JSON")
                },
                serde_json::error::Category::Io => {
                    write!(fmt, "Encountered an Input/Output error while handling JSON")
                },
            },
            FError::OptionError => write!(fmt, "Could not access an expected value"),
            FError::Quit { message } => write!(fmt, "{}", message),
            FError::RegexError => write!(fmt, "Failed to parse regular expression"),
            FError::ReqwestError { error }=> write!(fmt, "Failed to send/process an HTTP(S) request due to {}", error),
        }
    }
}

impl From<reqwest::Error> for FError {
    fn from(err: reqwest::Error) -> Self {
        Self::ReqwestError { error: err }
    }
}

impl From<fancy_regex::Error> for FError {
    fn from(_: fancy_regex::Error) -> Self {
        Self::RegexError
    }
}

impl From<std::io::Error> for FError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError {
            description: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for FError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonError {
            category: err.classify(),
        }
    }
}
