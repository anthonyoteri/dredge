/*
 * Copyright 2023 Anthony Oteri
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use std::time::Duration;

use reqwest::header;
use reqwest::header::HeaderValue;
use reqwest::StatusCode;
use serde::Deserialize;
use url::Url;

use crate::error::ApiError;

/// The MIME type for Docker Image Manifest V2, Schema 2.
///
/// This value is sent in `Accept` headers when fetching manifests so that
/// the registry returns the canonical V2 manifest rather than a legacy V1
/// manifest.
const MANIFEST_V2: &str = "application/vnd.docker.distribution.manifest.v2+json";

/// Connect timeout applied when establishing a TCP connection.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Overall request timeout from first byte sent to last byte received.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Build a shared [`reqwest::Client`] with sensible default timeouts.
///
/// All outbound HTTP requests should use this client to prevent hung
/// connections from blocking the process indefinitely.
///
/// # Errors
///
/// Returns [`ApiError::HttpError`] if the underlying TLS backend fails to
/// initialise and the client cannot be constructed.
pub fn build_client() -> Result<reqwest::Client, ApiError> {
    reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(ApiError::HttpError)
}

/// Fetch all pages of a paginated Docker Registry API endpoint and return the
/// collected, deserialized response bodies.
///
/// The Docker Registry HTTP API V2 paginates list responses using a `Link`
/// response header whose value is an [RFC 5988](https://tools.ietf.org/html/rfc5988)
/// URL pointing to the next page.  This function follows every `Link` header
/// until no further pages remain, accumulating each page's deserialized JSON
/// body into the returned `Vec<T>`.
///
/// # Arguments
///
/// * `client` — A configured [`reqwest::Client`] used to send requests.
/// * `origin` — The base URL of the Docker Registry (e.g.
///   `https://registry.example.com`).
/// * `path` — The API path to request (e.g. `v2/_catalog`).
///
/// # Errors
///
/// Returns an [`ApiError`] in any of the following situations:
///
/// * [`ApiError::UrlParseError`] — `origin` and `path` cannot be joined into a
///   valid URL.
/// * [`ApiError::HttpError`] — an HTTP request fails at the transport layer, or
///   a response body cannot be deserialized as JSON into `T`.
/// * [`ApiError::ResponseHeaderParseError`] — a `Link` header value contains
///   non-UTF-8 bytes.
/// * Any variant returned by [`parse_response_status`] — see that function for
///   the full list of status-code error conditions.
pub async fn fetch_paginated<T: for<'de> Deserialize<'de>>(
    client: &reqwest::Client,
    origin: &Url,
    path: &str,
) -> Result<Vec<T>, ApiError> {
    log::trace!("fetch_paginated(origin: {origin:?}, path: {path:?})");

    let mut responses: Vec<T> = Vec::default();
    let mut next_path = String::from(path);
    loop {
        let url = origin.join(&next_path)?;

        let resp = client.get(url).send().await?;
        parse_response_status(&resp)?;

        let headers = resp.headers().clone();

        responses.push(resp.json().await?);

        if let Some(p) = parse_rfc5988(headers.get(header::LINK))? {
            next_path = p;
        } else {
            break;
        }
    }
    Ok(responses)
}

/// Extract the URL from an optional RFC 5988 `Link` header value.
///
/// The Docker Registry API uses `Link` headers of the form
/// `<URL>; rel="next"` to signal the next page of a paginated result.
/// This function extracts the URL between the angle brackets from the
/// portion before the first `;`.
///
/// Returns `Ok(Some(url))` when a valid bracketed URL is found,
/// `Ok(None)` when the header is absent or does not contain a
/// bracketed URL (e.g. it is malformed or uses a different format).
///
/// # Errors
///
/// Returns an [`ApiError`] if the header value contains non-UTF-8 bytes.
fn parse_rfc5988(header_value: Option<&HeaderValue>) -> Result<Option<String>, ApiError> {
    log::trace!("parse_rfc5988(header_value: {header_value:?})");

    let Some(link_value) = header_value else {
        return Ok(None);
    };

    let link_str = link_value.to_str()?;
    // RFC 5988 link header format: `<URL>; rel="next"` — take everything
    // before the first ';', strip the surrounding angle brackets.
    let url_part = link_str.split_once(';').map_or(link_str, |(url, _)| url);
    let path = url_part
        .trim()
        .strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'));

    Ok(path.map(String::from))
}

/// Check that the `Docker-Distribution-API-Version` response header is present
/// and equals `"registry/2.0"`.
///
/// Returns `Ok(())` when the header is correct.
///
/// # Errors
///
/// * [`ApiError::ResponseHeaderParseError`] — the header value contains
///   non-UTF-8 bytes.
/// * [`ApiError::UnsupportedVersion`] — the header is present but its value
///   is not `"registry/2.0"`.
/// * [`ApiError::UnexpectedResponse`] — the header is entirely absent.
fn check_api_version_header(response: &reqwest::Response) -> Result<(), ApiError> {
    match response.headers().get("Docker-Distribution-API-Version") {
        Some(v) if v.to_str()? == "registry/2.0" => Ok(()),
        Some(v) => Err(ApiError::UnsupportedVersion(v.to_str()?.into())),
        None => Err(ApiError::UnexpectedResponse(
            "Missing version header".into(),
        )),
    }
}

/// Validate the HTTP status code of a Docker Registry API response.
///
/// The Docker Registry API contract requires that `2xx` responses include a
/// `Docker-Distribution-API-Version: registry/2.0` header.  `401 Unauthorized`
/// responses must also carry this header; when they do the caller should
/// authenticate and retry.  All other non-success codes are treated as errors.
///
/// # Errors
///
/// * [`ApiError::ResponseHeaderParseError`] — the `Docker-Distribution-API-Version`
///   header value contains non-UTF-8 bytes (only checked on `2xx` and `401`).
/// * [`ApiError::UnsupportedVersion`] — a `2xx` or `401` response contains the
///   version header with a value other than `"registry/2.0"`.
/// * [`ApiError::UnexpectedResponse`] — a `2xx` or `401` response is missing the
///   version header entirely.
/// * [`ApiError::AuthorizationFailed`] — the status code is `401 Unauthorized`
///   and the version header is valid.
/// * [`ApiError::NotFound`] — the status code is `404 Not Found`.
/// * [`ApiError::MethodNotAllowed`] — the status code is `405 Method Not Allowed`.
/// * [`ApiError::UnexpectedResponse`] — any other undocumented status code is
///   received.
pub fn parse_response_status(response: &reqwest::Response) -> Result<(), ApiError> {
    log::trace!("parse_response_status(response: {response:?})");

    match response.status() {
        StatusCode::OK | StatusCode::ACCEPTED => check_api_version_header(response),
        StatusCode::UNAUTHORIZED => {
            check_api_version_header(response)?;
            Err(ApiError::AuthorizationFailed)
        }
        StatusCode::NOT_FOUND => Err(ApiError::NotFound),
        StatusCode::METHOD_NOT_ALLOWED => Err(ApiError::MethodNotAllowed),
        e => Err(ApiError::UnexpectedResponse(format!(
            "Undocumented status code: {e:?}"
        ))),
    }
}

/// Fetch the content digest for the manifest at `url`.
///
/// Sends a `HEAD` request with an `Accept: application/vnd.docker.distribution.manifest.v2+json`
/// header and returns the value of the `docker-content-digest` response header.
/// This digest is required to delete a manifest, since the Docker Registry API
/// only accepts deletions by digest, not by tag name.
///
/// # Errors
///
/// * [`ApiError::HttpError`] — the HTTP request fails at the transport layer.
/// * [`ApiError::ResponseHeaderParseError`] — a response header contains
///   non-UTF-8 bytes.
/// * [`ApiError::UnexpectedResponse`] — the `docker-content-digest` header is
///   absent, or a `2xx` response is missing the version header.
/// * [`ApiError::UnsupportedVersion`] — the version header has an unexpected value.
/// * [`ApiError::AuthorizationFailed`] — the registry returns `401 Unauthorized`.
/// * [`ApiError::NotFound`] — the registry returns `404 Not Found`.
/// * [`ApiError::MethodNotAllowed`] — the registry returns `405 Method Not Allowed`.
pub async fn get_digest(client: &reqwest::Client, url: &Url) -> Result<String, ApiError> {
    log::trace!("get_digest(client: {client:?}, url: {url}");
    let resp = client
        .head(url.as_ref())
        .header(header::ACCEPT, MANIFEST_V2)
        .send()
        .await?;
    parse_response_status(&resp)?;

    let headers = resp.headers();
    Ok(String::from(
        headers
            .get("docker-content-digest")
            .ok_or(ApiError::UnexpectedResponse(String::from(
                "Missing docker-content-digest header",
            )))?
            .to_str()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test parsing a valid RFC5988 header value.
    ///
    /// Attempt to parse a valid RFC5988 header value, and ensure that the
    /// parsed URL was returned as expected.
    #[tokio::test]
    async fn test_parse_rfc5988_valid() {
        // Mock a valid RFC5988 header value
        let valid_header_value =
            HeaderValue::from_str(r#"<https://example.com/related>; rel="related""#)
                .expect("Failed to create valid header value");

        // Call the parse_rfc5988 function with the valid header value
        let result = parse_rfc5988(Some(&valid_header_value)).unwrap();

        // Assert that the function returned the expected URL as Some(String)
        assert_eq!(result, Some(String::from("https://example.com/related")));
    }

    /// Test parsing an invalid RFC5988 header value.
    ///
    /// Attempt to parse an invalid string as RFC5988, ensuring that the `None`
    /// variant is returned.
    #[tokio::test]
    async fn test_parse_rfc5988_invalid() {
        let invalid_header_value = HeaderValue::from_str(r"invalid header value")
            .expect("Failed to create valid header value");

        // Call the parse_rfc5988 function with the invalid header value
        let result = parse_rfc5988(Some(&invalid_header_value)).unwrap();

        // Assert that the function returned None
        assert_eq!(result, None);
    }

    /// Test that `parse_rfc5988` with `None` input returns `Ok(None)`.
    ///
    /// When no `Link` header is present in the response, the function should
    /// return `Ok(None)` to signal that there is no next page.
    #[test]
    fn test_parse_rfc5988_none_input() {
        let result = parse_rfc5988(None).unwrap();
        assert_eq!(result, None);
    }

    /// Validates the happy path for the `get_digest` function.
    ///
    /// This test starts up a mock server, and the client makes a request for
    /// the digest with the proper headers set.  The test then validates that
    /// the correct digest is returned and that the mock server had the expected
    /// interactions.
    #[tokio::test]
    async fn test_get_digest() -> Result<(), ApiError> {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/foo/manifests/latest";

        // Mock the HTTP response for the Docker Registry API
        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("HEAD", path)
            .match_header(http::header::ACCEPT.as_str(), MANIFEST_V2)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_header("Docker-Distribution-API-Version", "registry/2.0")
            .with_header(
                "docker-content-digest",
                "sha256:0259571889ac87efbfca5b79a0abe9baf626d058ec5f9a5744bace2229d9ed50",
            )
            .with_header(
                "etag",
                "sha256:0259571889ac87efbfca5b79a0abe9baf626d058ec5f9a5744bace2229d9ed50",
            )
            .create();

        let url = registry_url.join(path)?;
        let client = reqwest::Client::new();
        let result = get_digest(&client, &url).await;

        assert!(result.is_ok(), "{:?}", result.unwrap_err());
        assert_eq!(
            result.unwrap(),
            *"sha256:0259571889ac87efbfca5b79a0abe9baf626d058ec5f9a5744bace2229d9ed50"
        );

        mock_response.assert();

        Ok(())
    }

    /// Test `get_digest` when the `docker-content-digest` header is missing.
    ///
    /// The function must return `ApiError::UnexpectedResponse` when the registry
    /// omits the `docker-content-digest` header from an otherwise successful
    /// `HEAD` response.
    #[tokio::test]
    async fn test_get_digest_missing_digest_header() -> Result<(), Box<dyn std::error::Error>> {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/foo/manifests/latest";

        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("HEAD", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header("Docker-Distribution-API-Version", "registry/2.0")
            // No docker-content-digest header
            .create();

        let url = registry_url.join(path)?;
        let client = reqwest::Client::new();
        let result = get_digest(&client, &url).await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ApiError::UnexpectedResponse(_)),
            "Expected ApiError::UnexpectedResponse"
        );

        mock_response.assert();
        Ok(())
    }

    /// Test `fetch_paginated` happy path — single page with no `Link` header.
    ///
    /// When the registry returns a single page (no pagination link), the
    /// function should return a `Vec` containing exactly one parsed response.
    #[tokio::test]
    async fn test_fetch_paginated_single_page() -> Result<(), Box<dyn std::error::Error>> {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Resp {
            items: Vec<String>,
        }

        let mut server = mockito::Server::new_async().await;
        let path = "/v2/test/list";

        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("GET", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_header("Docker-Distribution-API-Version", "registry/2.0")
            .with_body(r#"{"items": ["a", "b", "c"]}"#)
            .create();

        let client = build_client().expect("Failed to build client");
        let result: Vec<Resp> = fetch_paginated(&client, &registry_url, path).await?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].items, vec!["a", "b", "c"]);

        mock_response.assert();
        Ok(())
    }

    /// Test that `fetch_paginated` propagates a JSON decode error on an empty body.
    ///
    /// When the registry returns a success status but no body, the JSON
    /// deserializer will fail.  The error must be surfaced to the caller rather
    /// than silently swallowed.
    #[tokio::test]
    async fn test_fetch_paginated_empty_body_returns_error() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Resp {
            #[allow(dead_code)]
            items: Vec<String>,
        }

        let mut server = mockito::Server::new_async().await;
        let path = "/v2/test/empty";

        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        server
            .mock("GET", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header("Docker-Distribution-API-Version", "registry/2.0")
            // No body — JSON deserialisation must fail and be propagated.
            .create();

        let client = build_client().expect("Failed to build client");
        let result: Result<Vec<Resp>, _> = fetch_paginated(&client, &registry_url, path).await;
        assert!(
            result.is_err(),
            "Expected an error on empty body but got Ok"
        );
    }

    /// Test `parse_response_status` with `UNAUTHORIZED` and valid version header
    /// returns `ApiError::AuthorizationFailed`.
    #[tokio::test]
    async fn test_parse_response_status_unauthorized_valid_version() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/";

        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        server
            .mock("GET", path)
            .with_status(http::status::StatusCode::UNAUTHORIZED.as_u16().into())
            .with_header("Docker-Distribution-API-Version", "registry/2.0")
            .create();

        let url = registry_url.join(path).expect("Failed to join URL");
        let resp = reqwest::get(url).await.expect("Request failed");
        let result = parse_response_status(&resp);

        assert!(
            matches!(result, Err(ApiError::AuthorizationFailed)),
            "Expected AuthorizationFailed, got {result:?}"
        );
    }

    /// Test `parse_response_status` with `UNAUTHORIZED` and wrong version header
    /// returns `ApiError::UnsupportedVersion`.
    #[tokio::test]
    async fn test_parse_response_status_unauthorized_wrong_version() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/";

        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        server
            .mock("GET", path)
            .with_status(http::status::StatusCode::UNAUTHORIZED.as_u16().into())
            .with_header("Docker-Distribution-API-Version", "registry/1.0")
            .create();

        let url = registry_url.join(path).expect("Failed to join URL");
        let resp = reqwest::get(url).await.expect("Request failed");
        let result = parse_response_status(&resp);

        assert!(
            matches!(result, Err(ApiError::UnsupportedVersion(_))),
            "Expected UnsupportedVersion, got {result:?}"
        );
    }

    /// Test `parse_response_status` with `UNAUTHORIZED` and missing version header
    /// returns `ApiError::UnexpectedResponse`.
    #[tokio::test]
    async fn test_parse_response_status_unauthorized_missing_version() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/";

        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        server
            .mock("GET", path)
            .with_status(http::status::StatusCode::UNAUTHORIZED.as_u16().into())
            // No Docker-Distribution-API-Version header
            .create();

        let url = registry_url.join(path).expect("Failed to join URL");
        let resp = reqwest::get(url).await.expect("Request failed");
        let result = parse_response_status(&resp);

        assert!(
            matches!(result, Err(ApiError::UnexpectedResponse(_))),
            "Expected UnexpectedResponse, got {result:?}"
        );
    }

    /// Test `parse_response_status` with `NOT_FOUND` returns `ApiError::NotFound`.
    #[tokio::test]
    async fn test_parse_response_status_not_found() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/";

        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        server
            .mock("GET", path)
            .with_status(http::status::StatusCode::NOT_FOUND.as_u16().into())
            .create();

        let url = registry_url.join(path).expect("Failed to join URL");
        let resp = reqwest::get(url).await.expect("Request failed");
        let result = parse_response_status(&resp);

        assert!(
            matches!(result, Err(ApiError::NotFound)),
            "Expected NotFound, got {result:?}"
        );
    }

    /// Test `parse_response_status` with `METHOD_NOT_ALLOWED` returns
    /// `ApiError::MethodNotAllowed`.
    #[tokio::test]
    async fn test_parse_response_status_method_not_allowed() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/";

        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        server
            .mock("GET", path)
            .with_status(http::status::StatusCode::METHOD_NOT_ALLOWED.as_u16().into())
            .create();

        let url = registry_url.join(path).expect("Failed to join URL");
        let resp = reqwest::get(url).await.expect("Request failed");
        let result = parse_response_status(&resp);

        assert!(
            matches!(result, Err(ApiError::MethodNotAllowed)),
            "Expected MethodNotAllowed, got {result:?}"
        );
    }

    /// Test `parse_response_status` with an unexpected status code returns
    /// `ApiError::UnexpectedResponse`.
    #[tokio::test]
    async fn test_parse_response_status_unexpected_status() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/";

        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        server
            .mock("GET", path)
            .with_status(
                http::status::StatusCode::INTERNAL_SERVER_ERROR
                    .as_u16()
                    .into(),
            )
            .create();

        let url = registry_url.join(path).expect("Failed to join URL");
        let resp = reqwest::get(url).await.expect("Request failed");
        let result = parse_response_status(&resp);

        assert!(
            matches!(result, Err(ApiError::UnexpectedResponse(_))),
            "Expected UnexpectedResponse, got {result:?}"
        );
    }
}
