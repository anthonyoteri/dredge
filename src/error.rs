#![allow(clippy::enum_variant_names)]

use thiserror::Error;

/// The common error type for this Application.
#[derive(Error, Debug)]
pub enum DredgeError {
    /// An error related to the configuration of the program.
    #[error(transparent)]
    ConfigError(#[from] ConfigError),

    /// An error communicating with the Registry API
    #[error(transparent)]
    ApiError(#[from] ApiError),
}

/// An error related to the configuration fo the program.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// An error parsing the configuration from disk.
    #[error("Failed to parse configuration file")]
    ParseError(Box<dyn std::error::Error>),

    /// An error writing the configuration to disk.
    #[error("Failed to write configuration data")]
    WriteError(Box<dyn std::error::Error>),

    /// A generic IOError
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

impl From<toml::ser::Error> for ConfigError {
    fn from(other: toml::ser::Error) -> Self {
        Self::WriteError(Box::from(other))
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(other: toml::de::Error) -> Self {
        Self::ParseError(Box::from(other))
    }
}

impl From<xdg::BaseDirectoriesError> for ConfigError {
    fn from(other: xdg::BaseDirectoriesError) -> Self {
        Self::WriteError(Box::from(other))
    }
}

/// An error related to the communication with the registry API.
#[derive(Error, Debug)]
pub enum ApiError {
    /// Error parsing a URL
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),

    /// Error in HTTP Request
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),

    #[error("Failed to parse response headers")]
    ResponseHeaderParseError(Box<dyn std::error::Error>),

    #[error("Version Mismatch {0}")]
    UnsupportedVersion(String),

    #[error("Unexpected response from API: {0}")]
    UnexpectedResponse(String),

    #[error("HTTP Authorization failed")]
    AuthorizationFailed,

    #[error("Resource not found")]
    NotFound,
}

impl From<reqwest::header::ToStrError> for ApiError {
    fn from(other: reqwest::header::ToStrError) -> Self {
        Self::ResponseHeaderParseError(Box::from(other))
    }
}
