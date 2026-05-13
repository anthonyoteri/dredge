/*
 * Copyright 2023 Anthony Oteri
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;

/// Command-line interface for `dredge`.
///
/// `dredge` is a tool for interacting with Docker Registry HTTP API V2
/// endpoints.  It supports listing repositories and tags, inspecting image
/// manifests, deleting tagged images, and verifying registry API
/// compatibility.
///
/// The `<REGISTRY>` positional argument accepts a bare hostname
/// (`registry.example.com`), a host-and-port pair
/// (`registry.example.com:5000`), or a full URL with an explicit scheme
/// (`https://registry.example.com` or `http://registry.example.com`).
/// When no scheme is given, `https://` is assumed.
#[derive(Debug, Parser, PartialEq, Eq)]
#[command(name = "dredge", version, author)]
#[command(about, long_about)]
pub(crate) struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,

    /// Minimum log level for messages written to stderr.
    ///
    /// Possible values: `trace`, `debug`, `info`, `warn`, `error`, `off`.
    /// Defaults to `info`.
    #[arg(
    long = "log-level",
    require_equals = true,
    value_name = "LEVEL",
    num_args = 0..=1,
    default_value_t = LogLevel::Info,
    default_missing_value = "info",
    value_enum
    )]
    pub log_level: LogLevel,

    /// The Docker Registry endpoint.
    ///
    /// Accepts a hostname (`registry.example.com`), host and port
    /// (`registry.example.com:5000`), or a full URL with an explicit scheme
    /// (`https://registry.example.com` or `http://registry.example.com`).
    /// The `https://` scheme is assumed when no scheme is provided.
    pub registry: String,
}

/// Log verbosity level for the `--log-level` CLI option.
///
/// Maps directly to the corresponding [`log::LevelFilter`] variants.  Use
/// `Off` to suppress all log output.
#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum LogLevel {
    /// Extremely verbose output, including internal trace points.
    Trace,
    /// Verbose output useful for debugging.
    Debug,
    /// Informational messages (default).
    Info,
    /// Warnings about potentially unexpected conditions.
    Warn,
    /// Only error messages.
    Error,
    /// Suppress all log output.
    Off,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(lvl: LogLevel) -> Self {
        match lvl {
            LogLevel::Trace => Self::Trace,
            LogLevel::Debug => Self::Debug,
            LogLevel::Info => Self::Info,
            LogLevel::Warn => Self::Warn,
            LogLevel::Error => Self::Error,
            LogLevel::Off => Self::Off,
        }
    }
}

/// Available `dredge` subcommands.
#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum Commands {
    /// List all repositories available in the registry catalog.
    ///
    /// Queries the `/v2/_catalog` endpoint and prints one repository name per
    /// line.  Paginated responses are followed automatically.
    ///
    /// **Example:**
    /// ```text
    /// dredge registry.example.com catalog
    /// ```
    Catalog,

    /// List all tags published for an image.
    ///
    /// Queries the `/v2/<NAME>/tags/list` endpoint and prints one tag per
    /// line.  Paginated responses are followed automatically.
    ///
    /// **Example:**
    /// ```text
    /// dredge registry.example.com tags myorg/backend
    /// ```
    #[command(arg_required_else_help = true)]
    Tags {
        /// The repository name whose tags should be listed
        /// (e.g. `myorg/backend`).
        name: String,
    },

    /// Show detailed manifest information for a tagged image.
    ///
    /// Queries the `/v2/<IMAGE>/manifests/<TAG>` endpoint and prints the
    /// parsed manifest as YAML, including the image name, tag, architecture,
    /// filesystem layers, content digest, and `ETag`.
    ///
    /// When `[TAG]` is omitted, `latest` is used.
    ///
    /// **Examples:**
    /// ```text
    /// dredge registry.example.com show myorg/backend
    /// dredge registry.example.com show myorg/backend v2.0.0
    /// ```
    #[command(arg_required_else_help = true)]
    Show {
        /// The repository name of the image to inspect (e.g. `myorg/backend`).
        image: String,
        /// The tag to inspect.  Defaults to `latest` when omitted.
        #[arg(default_missing_value = "latest")]
        tag: Option<String>,
    },

    /// Delete a tagged image from the registry.
    ///
    /// Resolves the tag to its content digest via a `HEAD` request, then
    /// sends a `DELETE` request for that digest to the
    /// `/v2/<IMAGE>/manifests/<DIGEST>` endpoint.
    ///
    /// Requires the registry to have storage deletion enabled (set
    /// `REGISTRY_STORAGE_DELETE_ENABLED=true` on the registry container).
    /// If deletion is not enabled the registry returns a
    /// `405 Method Not Allowed` response.
    ///
    /// Only the manifest is removed; unreferenced layer blobs remain on disk
    /// until the registry garbage collector is run.
    ///
    /// **Example:**
    /// ```text
    /// dredge registry.example.com delete myorg/backend v1.0.0
    /// ```
    #[command(arg_required_else_help = true)]
    Delete {
        /// The repository name of the image to delete (e.g. `myorg/backend`).
        image: String,
        /// The tag to delete (e.g. `v1.0.0`).
        tag: String,
    },

    /// Verify that the registry endpoint implements Docker Distribution API v2.
    ///
    /// Sends a `GET` request to `/v2` and checks that the response contains a
    /// `Docker-Distribution-API-Version: registry/2.0` header.  Prints `Ok`
    /// on success.
    ///
    /// **Example:**
    /// ```text
    /// dredge registry.example.com check
    /// ```
    Check,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that given the --log-level option, ensure that the corresponding
    /// `LogLevel` variant is set.
    #[test]
    fn test_log_level_option_off() {
        let args = vec!["dredge", "--log-level=off", "registry.local", "check"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.log_level, LogLevel::Off);
    }

    /// Test that given the --log-level option, ensure that the corresponding
    /// `LogLevel` variant is set.
    #[test]
    fn test_log_level_option_trace() {
        let args = vec!["dredge", "--log-level=trace", "registry.local", "check"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.log_level, LogLevel::Trace);
    }

    /// Test that given the --log-level option, ensure that the corresponding
    /// `LogLevel` variant is set.
    #[test]
    fn test_log_level_option_debug() {
        let args = vec!["dredge", "--log-level=debug", "registry.local", "check"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.log_level, LogLevel::Debug);
    }

    /// Test that given the --log-level option, ensure that the corresponding
    /// `LogLevel` variant is set.
    #[test]
    fn test_log_level_option_info() {
        let args = vec!["dredge", "--log-level=info", "registry.local", "check"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.log_level, LogLevel::Info);
    }

    /// Test that given the --log-level option, ensure that the corresponding
    /// `LogLevel` variant is set.
    #[test]
    fn test_log_level_option_warn() {
        let args = vec!["dredge", "--log-level=warn", "registry.local", "check"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.log_level, LogLevel::Warn);
    }

    /// Test that given the --log-level option, ensure that the corresponding
    /// `LogLevel` variant is set.
    #[test]
    fn test_log_level_option_error() {
        let args = vec!["dredge", "--log-level=error", "registry.local", "check"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.log_level, LogLevel::Error);
    }

    /// Test that given the <REGISTRY> argument and the "catalog" command,
    /// ensure that the expected values are received.
    #[test]
    fn test_catalog_command() {
        let args = vec!["dredge", "registry.local", "catalog"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.registry, String::from("registry.local"));
        assert_eq!(cli.command, Commands::Catalog);
    }

    /// Test that given the <REGISTRY> argument and the "tags" command with a
    /// specific image name, the expected values are received.
    #[test]
    fn test_tags_command() {
        let args = vec!["dredge", "registry.local", "tags", "foobar"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.registry, *"registry.local");
        assert_eq!(
            cli.command,
            Commands::Tags {
                name: String::from("foobar")
            }
        );
    }

    /// Test that given the <REGSITRY> argument and the "show" command with
    /// an image name but no tag, the expected values are received.
    #[test]
    fn test_show_command() {
        let args = vec!["dredge", "registry.local", "show", "foo"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.registry, *"registry.local");
        assert_eq!(
            cli.command,
            Commands::Show {
                image: String::from("foo"),
                tag: None,
            }
        );
    }

    /// Test that given the <REGSITRY> argument and the "show" command with
    /// both an image and tag, the expected values are received.
    #[test]
    fn test_show_command_with_optional_tag() {
        let args = vec!["dredge", "registry.local", "show", "foo", "bar"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.registry, *"registry.local");
        assert_eq!(
            cli.command,
            Commands::Show {
                image: String::from("foo"),
                tag: Some(String::from("bar")),
            }
        );
    }

    /// Test that given the <REGISTRY> argument and the "delete" command, with
    /// both an image and tag, the expected values are received.
    #[test]
    fn test_delete_command() {
        let args = vec!["dredge", "registry.local", "delete", "foo", "bar"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.registry, *"registry.local");
        assert_eq!(
            cli.command,
            Commands::Delete {
                image: String::from("foo"),
                tag: String::from("bar"),
            }
        );
    }

    /// Test that given the <REGISTRY> argument and the "check" command, the
    /// expected values are received.
    #[test]
    fn test_check_command() {
        let args = vec!["dredge", "registry.local", "check"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.registry, *"registry.local");
        assert_eq!(cli.command, Commands::Check);
    }
}
