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

#![deny(clippy::pedantic)]

use clap::Parser;
use url::Url;

use crate::cli::Cli;
use crate::cli::Commands;

use crate::error::DredgeError;

mod api;
pub(crate) mod cli;
mod commands;
mod error;

/// Name of "latest" tag
const LATEST: &str = "latest";

/// Parse the "<REGISTRY>" argument into a complete  Docker Registry URL.
///
/// This prepends the HTTPS scheme and converts the given string to a `Url`
/// instance.
///
/// If the given `host` value is already a valid URL, then it will be returned
/// as-is.
///
/// # Errors:
///
/// If there is a problem parsing the resulting string as a valid URL, a
/// `DredgeError::RegistryUrlError` will be returned.
fn parse_registry_arg(host: &str) -> Result<Url, DredgeError> {
    log::trace!("make_registry_url(host: {host})");

    Url::parse(host)
        .or_else(|_| Url::parse(&format!("https://{host}")))
        .or(Err(DredgeError::RegistryUrlError(host.to_string())))
}

#[async_std::main]
async fn main() -> Result<(), DredgeError> {
    let args = Cli::parse();

    // -- Initialize logging
    let log_level = args.log_level;
    femme::with_level(log::LevelFilter::from(log_level));

    // -- Parse the given <REGISTRY> argument into a complete URL
    let registry_url: Url = parse_registry_arg(&args.registry)?;

    // -- Dispatch control to the appropriate command handler.
    match args.command {
        Commands::Catalog => commands::catalog_handler(&registry_url).await?,
        Commands::Tags { name } => commands::tags_handler(&registry_url, &name).await?,
        Commands::Show { image, tag } => {
            commands::show_handler(&registry_url, &image, &tag.unwrap_or(LATEST.to_string()))
                .await?;
        }
        Commands::Delete { image, tag } => {
            commands::delete_handler(&registry_url, &image, &tag).await?;
        }
        Commands::Check => commands::check_handler(&registry_url).await?,
    }

    Ok(())
}
