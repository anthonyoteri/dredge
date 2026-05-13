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

/// The top-level error type for the `dredge` application.
///
/// Wraps lower-level errors from the registry API, URL parsing, I/O, and
/// the logging subsystem so they can all be returned from `main`.
#[derive(Error, Debug)]
pub enum DredgeError {
    /// An error returned by the Docker Registry API layer.
    #[error(transparent)]
    ApiError(#[from] ApiError),

    /// The `<REGISTRY>` argument could not be parsed as a valid URL.
    #[error("Error determining registry URL from {0}")]
    RegistryUrlError(String),

    /// An I/O error writing output to stdout.
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// The logging subsystem could not be initialised.
    #[error(transparent)]
    LoggerError(#[from] log::SetLoggerError),
}

/// Errors that can occur while communicating with the Docker Registry API.
#[derive(Error, Debug)]
pub enum ApiError {
    /// A URL could not be constructed or parsed.
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),

    /// An HTTP transport-level error (connection refused, TLS failure, timeout,
    /// etc.) or a response body that could not be decoded.
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),

    /// A response header contained bytes that could not be decoded as UTF-8.
    #[error("Failed to parse response header: {0}")]
    ResponseHeaderParseError(String),

    /// The registry returned a `Docker-Distribution-API-Version` header value
    /// other than `"registry/2.0"`.  The inner `String` holds the actual value.
    #[error("Version Mismatch {0}")]
    UnsupportedVersion(String),

    /// The registry returned a response that did not match the expected API
    /// contract (e.g. a required header was absent).  The inner `String`
    /// describes the specific problem.
    #[error("Unexpected response from API: {0}")]
    UnexpectedResponse(String),

    /// The registry returned `401 Unauthorized`.  Authentication is not
    /// currently supported; the request cannot be retried automatically.
    #[error("HTTP Authorization failed")]
    AuthorizationFailed,

    /// The requested resource does not exist in the registry (`404 Not Found`).
    #[error("Resource not found")]
    NotFound,

    /// An I/O error writing serialized output to the output buffer.
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// The manifest response body could not be serialized to YAML for output.
    #[error(transparent)]
    SerializerError(#[from] serde_norway::Error),

    /// The registry returned `405 Method Not Allowed`, typically because
    /// storage deletion has not been enabled on the registry.
    #[error("Method not allowed")]
    MethodNotAllowed,
}

impl From<reqwest::header::ToStrError> for ApiError {
    fn from(other: reqwest::header::ToStrError) -> Self {
        Self::ResponseHeaderParseError(other.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that `DredgeError::from(ApiError::NotFound)` works via the `From` impl.
    #[test]
    fn test_dredge_error_from_api_error_not_found() {
        let api_err = ApiError::NotFound;
        let dredge_err = DredgeError::from(api_err);
        assert!(matches!(
            dredge_err,
            DredgeError::ApiError(ApiError::NotFound)
        ));
    }

    /// Test that `DredgeError::from(ApiError::AuthorizationFailed)` works.
    #[test]
    fn test_dredge_error_from_api_error_authorization_failed() {
        let api_err = ApiError::AuthorizationFailed;
        let dredge_err = DredgeError::from(api_err);
        assert!(matches!(
            dredge_err,
            DredgeError::ApiError(ApiError::AuthorizationFailed)
        ));
    }

    /// Test that `DredgeError::from(ApiError::MethodNotAllowed)` works.
    #[test]
    fn test_dredge_error_from_api_error_method_not_allowed() {
        let api_err = ApiError::MethodNotAllowed;
        let dredge_err = DredgeError::from(api_err);
        assert!(matches!(
            dredge_err,
            DredgeError::ApiError(ApiError::MethodNotAllowed)
        ));
    }

    /// Test Display output for `ApiError::NotFound`.
    #[test]
    fn test_api_error_not_found_display() {
        let err = ApiError::NotFound;
        assert_eq!(err.to_string(), "Resource not found");
    }

    /// Test Display output for `ApiError::AuthorizationFailed`.
    #[test]
    fn test_api_error_authorization_failed_display() {
        let err = ApiError::AuthorizationFailed;
        assert_eq!(err.to_string(), "HTTP Authorization failed");
    }

    /// Test Display output for `ApiError::MethodNotAllowed`.
    #[test]
    fn test_api_error_method_not_allowed_display() {
        let err = ApiError::MethodNotAllowed;
        assert_eq!(err.to_string(), "Method not allowed");
    }

    /// Test Display output for `ApiError::UnsupportedVersion`.
    #[test]
    fn test_api_error_unsupported_version_display() {
        let err = ApiError::UnsupportedVersion(String::from("registry/1.0"));
        assert_eq!(err.to_string(), "Version Mismatch registry/1.0");
    }

    /// Test Display output for `ApiError::UnexpectedResponse`.
    #[test]
    fn test_api_error_unexpected_response_display() {
        let err = ApiError::UnexpectedResponse(String::from("Missing header"));
        assert_eq!(
            err.to_string(),
            "Unexpected response from API: Missing header"
        );
    }

    /// Test Display output for `DredgeError::RegistryUrlError`.
    #[test]
    fn test_dredge_error_registry_url_error_display() {
        let err = DredgeError::RegistryUrlError(String::from("bad-url"));
        assert_eq!(
            err.to_string(),
            "Error determining registry URL from bad-url"
        );
    }
}
