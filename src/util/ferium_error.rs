use std::fmt::Debug;
use thiserror::Error;

pub type FResult<T> = std::result::Result<T, FError>;

#[derive(Error, Debug)]
pub enum FError {
    /// The config file does not contain mods or repos
    #[error("Your config file is empty! Run `ferium help` to see how to add mods or repositories")]
    EmptyConfigFile,
    /// An HTTP(S) request returned with an error
    #[error("Failed to send/process an HTTP(S) request due to {}", .0)]
    ReqwestError(#[from] reqwest::Error),
    /// Failure to unwrap an Option, akin to `NullPointerError`s
    #[error("Could not access an expected value")]
    OptionError,
    /// Failed to parse a regular expression
    #[error("Failed to parse regular expression")]
    RegexError(#[from] fancy_regex::Error),
    /// A JSON error occured
    #[error("{}", match .0.classify() {
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
    JsonError(#[from] serde_json::Error),
    /// An HTTP(S) request encountered an error
    #[error("An HTTP(S) request returned an error, {}", message)]
    HTTPError { message: String },
    /// An I/O error occured
    #[error("Encountered an input/output error, {}", .0.to_string())]
    IOError(#[from] std::io::Error),
    /// The program is running on an unsupported device
    #[error("The device you are currently running on is unsupported by Ferium")]
    InvalidDeviceError,
    /// The application should print `message` and quit (gracefully)
    #[error("{}", message)]
    Quit { message: String },
}
