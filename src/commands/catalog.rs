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

//! Command module responsible for handling the "catalog" command.
//!
//! The "catalog" command works with the Docker Registry APIs "catalog"
//! entity available at /v2/_catalog.
//!
use serde::Deserialize;

use crate::api;
use crate::config::Config;
use crate::error::ApiError;

/// Path to the Docker Registry APIs "catalog" entity.
const BASE_CATALOG_URI: &str = "/v2/_catalog";

/// Handler for the `Catalog` endpoint
///
/// Fetch the list of repository names from the Docker Registry API, and
/// simply print the resulting names to stdout.
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem fetching or parsing the
/// responses from the Docker Registry API.  
pub async fn handler(config: &Config) -> Result<(), ApiError> {
    #[derive(Deserialize)]
    struct Response {
        repositories: Vec<String>,
    }

    log::trace!("handler()");

    let responses: Vec<Response> = api::fetch_all(config, BASE_CATALOG_URI).await?;
    let repository_list: Vec<&str> = responses
        .iter()
        .flat_map(|r| r.repositories.iter().map(String::as_str))
        .collect();

    for repository in repository_list {
        println!("{repository}");
    }

    Ok(())
}
