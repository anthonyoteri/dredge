/*
 * Copyright 2023 Anthony Oteri
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

#![allow(clippy::enum_variant_names)]
#![allow(clippy::module_name_repetitions)]

use thiserror::Error;

/// The common error type for this Application.
#[derive(Error, Debug)]
pub enum DredgeError {
    /// An error communicating with the Registry API
    #[error(transparent)]
    ApiError(#[from] ApiError),

    /// An error building the registry URL
    #[error("Error determining registry URL from {0}")]
    RegistryUrlError(String),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    LoggerError(#[from] log::SetLoggerError),
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

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    SerializerError(#[from] serde_yaml::Error),

    #[error("Method not allowed")]
    MethodNotAllowed,
}

impl From<reqwest::header::ToStrError> for ApiError {
    fn from(other: reqwest::header::ToStrError) -> Self {
        Self::ResponseHeaderParseError(Box::from(other))
    }
}
