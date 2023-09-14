//! Command module responsible for handling the API Version check.
//!
//! This is a minimal endpoint suitable for ensuring that the configured
//! Docker Regsitry API supports the correct API version.
//!
use crate::config::Config;
use crate::error::ApiError;

/// Path to the Docker Registry API's "api version check" endpoint.
const BASE_URL: &str = "/v2";

/// Handler for the API Version Check.
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem communicating with the
/// endpoint or if the required version is not supported.
pub async fn handler(config: &Config) -> Result<(), ApiError> {
    log::trace!("handler()");

    let url = config.registry_url.join(BASE_URL)?;
    let response = reqwest::get(url).await?;

    parse_response_status(&response)?;
    println!("Ok");
    Ok(())
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
fn parse_response_status(response: &reqwest::Response) -> Result<(), ApiError> {
    match response.status() {
        http::StatusCode::OK => {
            let headers = response.headers();
            if let Some(header_value) = headers.get("Docker-Distribution-API-Version") {
                if header_value.to_str()? != "registry/2.0" {
                    Err(ApiError::UnsupportedVersion(header_value.to_str()?.into()))
                } else {
                    Ok(())
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
                if header_value.to_str()? != "registry/2.0" {
                    Err(ApiError::UnsupportedVersion(header_value.to_str()?.into()))
                } else {
                    Err(ApiError::AuthorizationFailed)
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
