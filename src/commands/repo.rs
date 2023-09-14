//! Command module responsible for handling the "repo" command.
//!
//! The "repo" command works with the Docker Registry API's "repository"
//! entitity available at /v2/_catalog.
//!
use crate::cli::{RepoArgs, RepoCommands};
use crate::config::Config;
use crate::error::ApiError;
use serde::Deserialize;

/// Path to the Docker Registry API's "repository" entity.
const BASE_CATALOG_URI: &str = "/v2/_catalog?n=1000";

/// Main handler function for the "repo" command.
///
/// Responsible for dispatching the various subcommands.
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem handling the requested
/// subcommand.
pub async fn handler(config: &Config, args: &RepoArgs) -> Result<(), ApiError> {
    log::trace!("handler()");

    match args.command {
        RepoCommands::List => handle_list(config, args).await?,
    }

    Ok(())
}

/// Handle the repository list command.
///
/// Fetch the list of repository names from the Docker Registry API, and
/// simply print the resulting names to stdout.
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem fetching or parsing the
/// responses from the Docker Registry API.  
async fn handle_list(config: &Config, _args: &RepoArgs) -> Result<(), ApiError> {
    #[derive(Deserialize)]
    struct Response {
        repositories: Vec<String>,
    }

    log::trace!("handle_list()");

    let responses: Vec<Response> = fetch_all(config, BASE_CATALOG_URI).await?;
    let repo_list: Vec<&str> = responses
        .iter()
        .flat_map(|r| r.repositories.iter().map(String::as_str))
        .collect();

    for repo in repo_list {
        println!("{repo}");
    }

    Ok(())
}

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
async fn fetch_all<T: for<'de> serde::Deserialize<'de>>(
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
