use std::convert::From;

pub type FResult<T> = std::result::Result<T, FError>;

#[derive(Debug)]
pub enum FError {
    /// Error with file picker occured
    NativeDialogError,
    /// The config file does not contain mods or repos
    EmptyConfigFile,
    /// An HTTP(S) request returned with an error
    ReqwestError { error: reqwest::Error },
    /// Failed to unwrap an Option. Basically a `NullPointerError`
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

impl From<reqwest::Error> for FError {
    fn from(err: reqwest::Error) -> Self {
        Self::ReqwestError { error: err }
    }
}

impl From<native_dialog::Error> for FError {
    fn from(err: native_dialog::Error) -> Self {
        match err {
            native_dialog::Error::IoFailure(io_err) => Self::IOError {
                description: io_err.to_string(),
            },
            native_dialog::Error::NoImplementation => Self::InvalidDeviceError,
            _ => Self::NativeDialogError,
        }
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
