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

use std::io::{self, Write};

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

    let mut host = String::from(host);
    if !host.starts_with("http://") && !host.starts_with("https://") {
        host = format!("https://{host}");
    }

    Url::parse(&host).or(Err(DredgeError::RegistryUrlError(host.to_string())))
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
    let mut buf: Vec<u8> = Vec::new();
    match args.command {
        Commands::Catalog => commands::catalog_handler(&mut buf, &registry_url).await?,
        Commands::Tags { name } => commands::tags_handler(&mut buf, &registry_url, &name).await?,
        Commands::Show { image, tag } => {
            commands::show_handler(
                &mut buf,
                &registry_url,
                &image,
                &tag.unwrap_or(LATEST.to_string()),
            )
            .await?;
        }
        Commands::Delete { image, tag } => {
            commands::delete_handler(&mut buf, &registry_url, &image, &tag).await?;
        }
        Commands::Check => commands::check_handler(&mut buf, &registry_url).await?,
    }

    io::stdout().write_all(&buf)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that given a valid URL in the <REGISTRY> argument, we return the
    /// same URL from `parse_registry_arg()`
    #[test]
    fn test_parse_valid_url_registry_arg() {
        let host = "https://example.com/registry";
        let result = parse_registry_arg(host);

        // Check if the result is Ok and contains the expected URL
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.path(), "/registry");
    }

    /// Test that given only an FQDN for a specific host in the <REGISTRY>
    /// argument, we return an HTTPS url with that FQDN as the host.
    #[test]
    fn test_parse_valid_fqdn_registry_arg() {
        let host = "example.com";
        let result = parse_registry_arg(host);

        // Check if the result is Ok and contains the expected URL
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.path(), "/");
    }

    /// Test that given an FQDN with port for a specific host in the <REGISTRY>
    /// argument, we return an HTTPS url with that FQDN as the host and the
    /// given port as the parsed port number.
    #[test]
    fn test_parse_valid_fqdn_registry_arg_alt_port() {
        let host = "example.com:5123";
        let result = parse_registry_arg(host);

        // Check if the result is Ok and contains the expected URL
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.port(), Some(5123));
        assert_eq!(url.path(), "/");
    }

    /// Test that given an arbitrary string which can not be parsed as a valid
    /// URL or FQDN, we return the `RegistryUrlError` variant.
    #[test]
    fn test_parse_invalid_registry_arg() {
        let host = "///"; // This is not a valid URL
        let result = parse_registry_arg(host);

        // Check if result is Err and matches the expected error variant.
        assert!(result.is_err());
        match result {
            Err(DredgeError::RegistryUrlError(_)) => {} // Expected error variant,
            _ => panic!("Expected RegistryUrlError, got a different error"),
        }
    }
}
