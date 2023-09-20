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

use crate::api;
use crate::config::Config;
use crate::error::ApiError;

/// Handler for the `Catalog` endpoint
///
/// Fetch the list of repository names from the Docker Registry API, and
/// simply print the resulting names to stdout.
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem fetching or parsing the
/// responses from the Docker Registry API.  
pub async fn catalog_handler(config: &Config) -> Result<(), ApiError> {
    #[derive(Deserialize)]
    struct Response {
        repositories: Vec<String>,
    }

    log::trace!("catalog_handler()");
    let path = "v2/_catalog";

    let responses: Vec<Response> = api::fetch_all(config, path).await?;
    let repository_list: Vec<&str> = responses
        .iter()
        .flat_map(|r| r.repositories.iter().map(String::as_str))
        .collect();

    for repository in repository_list {
        println!("{repository}");
    }

    Ok(())
}

/// Handler for the `Tags` endpoint
///
/// Fetch the list of tags names for a given image from the Docker Registry API, and
/// simply print the resulting names to stdout.
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem fetching or parsing the
/// responses from the Docker Registry API.  
pub async fn tags_handler(config: &Config, name: &str) -> Result<(), ApiError> {
    #[derive(Deserialize)]
    struct Response {
        tags: Vec<String>,
    }

    log::trace!("tags_handler(name: {name})");
    let path = format!("/v2/{name}/tags/list");

    let responses: Vec<Response> = api::fetch_all(config, &path).await?;
    let tag_list: Vec<&str> = responses
        .iter()
        .flat_map(|r| r.tags.iter().map(String::as_str))
        .collect();

    for tag in tag_list {
        println!("{tag}");
    }

    Ok(())
}

/// Handler function for showing manifest details
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem fetching the manifest or if there
/// is a problem parsing the response from the Docker Registry API.
pub async fn show_handler(config: &Config, image: &str, tag: &str) -> Result<(), ApiError> {
    log::trace!("show_handler(image: {image}, tag: {tag})");
    let base = config.registry_url.to_owned();
    let path = format!("/v2/{image}/manifests/{tag}");
    let _url = base.join(&path)?;
    Ok(())
}

/// Handler function for deleting a manifest for a given tagged image.
///
/// # Errors:
///
/// Returns and `ApiError` if there is a problem converting the given tag to a
/// manifest digest, or if there is a problem deleting the manifest from the
/// Docker Registry API.
pub async fn delete_handler(_config: &Config, image: &str, tag: &str) -> Result<(), ApiError> {
    log::trace!("delete_handler(image: {image}, tag: {tag})");
    todo!()
}

// Path to the Docker Registry APIs "api version check" endpoint.

/// Handler for the API Version Check.
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem communicating with the
/// endpoint or if the required version is not supported.
pub async fn check_handler(config: &Config) -> Result<(), ApiError> {
    log::trace!("check_handler()");

    let base = config.registry_url.to_owned();
    let path = "/v2";
    let url = base.join(path)?;

    let response = reqwest::get(url).await?;

    api::parse_response_status(&response)?;
    println!("Ok");
    Ok(())
}
