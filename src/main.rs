/*
 * Copyright 2023 Anthony Oteri
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

#![deny(clippy::pedantic)]

use std::io::{self, Write};

use clap::Parser;
use simple_logger::SimpleLogger;
use url::Url;

use crate::cli::Cli;
use crate::cli::Commands;
use crate::error::DredgeError;

mod api;
pub(crate) mod cli;
mod commands;
mod error;

/// The default image tag used when no tag is specified by the caller.
const LATEST: &str = "latest";

/// Parse the `<REGISTRY>` CLI argument into a complete Docker Registry [`Url`].
///
/// Accepts a bare hostname (`registry.example.com`), a host-and-port pair
/// (`registry.example.com:5000`), or a full URL
/// (`https://registry.example.com:5000`).  When no URL scheme is present,
/// `https://` is prepended automatically before parsing.
///
/// # Errors
///
/// Returns [`DredgeError::RegistryUrlError`] containing the attempted URL
/// string if it cannot be parsed as a valid URL after the scheme is prepended.
///
/// # Examples
///
/// ```rust,ignore
/// // Bare hostname — HTTPS is assumed
/// let url = parse_registry_arg("registry.example.com").unwrap();
/// assert_eq!(url.scheme(), "https");
///
/// // Host with port
/// let url = parse_registry_arg("registry.example.com:5000").unwrap();
/// assert_eq!(url.port(), Some(5000));
///
/// // Full URL returned as-is
/// let url = parse_registry_arg("https://registry.example.com").unwrap();
/// assert_eq!(url.as_str(), "https://registry.example.com/");
/// ```
fn parse_registry_arg(host: &str) -> Result<Url, DredgeError> {
    log::trace!("parse_registry_arg(host: {host})");

    let mut host = String::from(host);
    if !host.starts_with("http://") && !host.starts_with("https://") {
        host = format!("https://{host}");
    }

    Url::parse(&host).or(Err(DredgeError::RegistryUrlError(host.clone())))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), DredgeError> {
    let args = Cli::parse();

    // -- Initialize logging
    let log_level = args.log_level;
    SimpleLogger::new()
        .with_colors(true)
        .with_utc_timestamps()
        .with_level(log_level.into())
        .env()
        .init()?;

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
                tag.as_deref().unwrap_or(LATEST),
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

    /// Test that an HTTP (non-HTTPS) URL is returned as-is without prepending
    /// the HTTPS scheme.
    #[test]
    fn test_parse_registry_arg_http_url() {
        let host = "http://example.com/registry";
        let result = parse_registry_arg(host);

        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.path(), "/registry");
    }

    /// Test that a trailing slash in the registry argument is preserved.
    #[test]
    fn test_parse_registry_arg_trailing_slash() {
        let host = "example.com/registry/";
        let result = parse_registry_arg(host);

        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.path(), "/registry/");
    }
}
