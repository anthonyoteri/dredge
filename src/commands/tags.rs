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

//! Command module responsible for handling the "tags" command.
//!
//! The "tags" command works with the Docker Registry API's "tags"
//! entitity available at /v2/<name>/tags/list.
//!
use serde::Deserialize;

use crate::api;
use crate::cli::TagsArgs;
use crate::config::Config;
use crate::error::ApiError;

/// Path to the Docker Registry API's "catalog" entity.
const BASE_TAGS_URI: &str = "/v2/{name}/tags/list";

/// Handler for the `Tags` endpoint
///
/// Fetch the list of tags names for a given image from the Docker Registry API, and
/// simply print the resulting names to stdout.
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem fetching or parsing the
/// responses from the Docker Registry API.  
pub async fn handler(config: &Config, args: &TagsArgs) -> Result<(), ApiError> {
    #[derive(Deserialize)]
    struct Response {
        name: String,
        tags: Vec<String>,
    }

    log::trace!("handler()");

    let name = args.name.clone();
    let url = BASE_TAGS_URI.replace("{name}", &name);
    let responses: Vec<Response> = api::fetch_all(config, &url).await?;
    let tag_list: Vec<&str> = responses
        .iter()
        .flat_map(|r| r.tags.iter().map(String::as_str))
        .collect();

    for tag in tag_list {
        println!("{tag}");
    }

    Ok(())
}
