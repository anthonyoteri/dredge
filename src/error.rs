/*
 *    Copyright 2023 Anthony Oteri
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
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
}

impl From<reqwest::header::ToStrError> for ApiError {
    fn from(other: reqwest::header::ToStrError) -> Self {
        Self::ResponseHeaderParseError(Box::from(other))
    }
}
