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

use serde::Deserialize;
use url::Url;

use crate::error::ApiError;

/// Iterate over a paginated result set, collecting and returning the response
/// set.
///
/// The Docker Registry API specifies that when making a GET request, the
/// response will be paginated using a Link response header for the Next URI.
/// The URL will be encoded using [RFC5988](https://tools.ietf.org/html/rfc5988)
///
/// This function will continuously request the "Next" link as long as it is
/// returned, collecting and returning the deserialized response bodies as a
/// Vec<T>.
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem constructing the URL from the
/// configured `registry_url` base and the given `path`, or if there is an
/// error deserializing the HTTP response body as JSON, or if there is an
/// error parsing the `Link` header value as an RFC5988 URL.
pub async fn fetch_paginated<T: for<'de> Deserialize<'de>>(
    origin: &Url,
    path: &str,
) -> Result<Vec<T>, ApiError> {
    log::trace!("fetch_paginated(origin: {origin:?}, path: {path:?})");

    let mut responses: Vec<T> = Vec::default();
    let mut next_path = String::from(path);
    loop {
        let url = origin.join(&next_path)?;

        let resp = reqwest::get(url).await?;
        let headers = resp.headers().clone();
        responses.push(resp.json().await?);

        if let Some(p) = parse_rfc5988(headers.get(http::header::LINK))? {
            next_path = p;
        } else {
            break;
        }
    }
    Ok(responses)
}

/// Given an optional header value possibly containing an RFC5988 formatted
/// URL, parse said URL into a `String`.
///
/// If the `header_value` does not contain a correctly formatted RFC5988 URL,
/// or if the `header_value` is not properly formatted containing a URL
/// surrounded by angle brackets, separated from the link relation by a ';'
/// character, the `None` variant will be returned.
///
/// # Errors:
///
/// Returns and `ApiError` if there is a problem parsing contents of the
/// supplied header value.
fn parse_rfc5988(header_value: Option<&http::HeaderValue>) -> Result<Option<String>, ApiError> {
    log::trace!("parse_rfc5988(header_value: {header_value:?})");

    if let Some(link_value) = header_value {
        let link_str = link_value.to_str()?;
        let parts: Vec<&str> = link_str.split(';').collect();
        if let Some(url_part) = parts.first() {
            if let Some(path) = url_part
                .trim()
                .strip_prefix('<')
                .and_then(|s| s.strip_suffix('>'))
            {
                return Ok(Some(String::from(path)));
            }
        }
    }

    Ok(None)
}

/// Parse the response according to the API Documentation.
///
/// If a 200 OK response is returned, the registry implements the V2(.1)
/// registry API and the client may proceed safely with other V2 operations.
/// Optionally, the response may contain information about the supported
/// paths in the response body. The client should be prepared to ignore this data.
///
/// If a 401 Unauthorized response is returned, the client should take action
/// based on the contents of the "WWW-Authenticate" header and try the endpoint
/// again. Depending on access control setup, the client may still have to
/// authenticate against different resources, even if this check succeeds.
///
/// If 404 Not Found response status, or other unexpected status, is returned,
/// the client should proceed with the assumption that the registry does not
/// implement V2 of the API.
///
/// When a 200 OK or 401 Unauthorized response is returned, the
/// "Docker-Distribution-API-Version" header should be set to "registry/2.0".
/// Clients may require this header value to determine if the endpoint serves
/// this API. When this header is omitted, clients may fallback to an older
/// API version.
///
/// # Errors:
///
/// Returns an `ApiError` on the following conditions:
///
/// * There is an error parsing the "Docker-Distribution-API-Version" header.
/// * The value of the above header is not the expected result.
/// * The above header is missing from the response.
/// * A non 200 HTTP response status code is returned.
pub fn parse_response_status(response: &reqwest::Response) -> Result<(), ApiError> {
    log::trace!("parse_response_status(response: {response:?})");

    match response.status() {
        http::StatusCode::OK => {
            let headers = response.headers();
            if let Some(header_value) = headers.get("Docker-Distribution-API-Version") {
                if header_value.to_str()? == "registry/2.0" {
                    Ok(())
                } else {
                    Err(ApiError::UnsupportedVersion(header_value.to_str()?.into()))
                }
            } else {
                Err(ApiError::UnexpectedResponse(
                    "Missing version header".into(),
                ))
            }
        }
        http::StatusCode::UNAUTHORIZED => {
            let headers = response.headers();
            if let Some(header_value) = headers.get("Docker-Distribution-API-Version") {
                if header_value.to_str()? == "registry/2.0" {
                    Err(ApiError::AuthorizationFailed)
                } else {
                    Err(ApiError::UnsupportedVersion(header_value.to_str()?.into()))
                }
            } else {
                Err(ApiError::UnexpectedResponse(
                    "Missing version header".into(),
                ))
            }
        }
        http::StatusCode::NOT_FOUND => Err(ApiError::NotFound),
        _ => Err(ApiError::UnexpectedResponse(
            "Undocumented status code".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use http::header::HeaderValue;

    use super::*;

    /// Test parsing a valid RFC5988 header value.
    ///
    /// Attempt to parse a valid RFC5988 header value, and ensure that the
    /// parsed URL was returned as expected.
    #[async_std::test]
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
    #[async_std::test]
    async fn test_parse_rfc5988_invalid() {
        // Mock a valid RFC5988 header value
        let invalid_header_value = HeaderValue::from_str(r#"invalid header value"#)
            .expect("Failed to create valid header value");

        // Call the parse_rfc5988 function with the valid header value
        let result = parse_rfc5988(Some(&invalid_header_value)).unwrap();

        // Assert that the function returned the expected URL as Some(String)
        assert_eq!(result, None);
    }
}
