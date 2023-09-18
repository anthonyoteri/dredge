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

use crate::config::Config;
use crate::error::ApiError;

/// Iterate over a paginated result set, collecting and returning the response
/// set.
///
/// The Docker Registry API specifies that when making a GET request, the
/// response will be paginated using a Link response header for the Next URI.
/// The URL will be encoded using RFC5988. [https://tools.ietf.org/html/rfc5988]
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
pub async fn fetch_all<T: for<'de> Deserialize<'de>>(
    config: &Config,
    path: &str,
) -> Result<Vec<T>, ApiError> {
    log::trace!("fetch_all({path:?})");

    let mut responses: Vec<T> = Vec::default();
    let mut uri = String::from(path);
    loop {
        log::debug!("GET {uri:?}");
        let url = config.registry_url.join(&uri)?;

        let resp = reqwest::get(url).await?;
        let headers = resp.headers().to_owned();
        responses.push(resp.json().await?);

        if let Some(path) = parse_rfc5988(headers.get(http::header::LINK))? {
            uri = path;
        } else {
            break;
        }
    }
    Ok(responses)
}

/// Given an optional header value possibly containing an RFC5988 formatted
/// URL, parse said URL into a `String`.
///
/// If the header_value does not contain a correctly formatted RFC5988 URL,
/// or if the header_value is not properly formatted containing a URL
/// surrounded by angle brackets, separated from the link relation by a ';'
/// character, the `None` variant will be returned.
///
/// # Errors:
///
/// Returns and `ApiError` if there is a problem parsing contents of the
/// supplied header value.
fn parse_rfc5988(header_value: Option<&http::HeaderValue>) -> Result<Option<String>, ApiError> {
    log::trace!("parse_rfc5988({header_value:?})");

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
