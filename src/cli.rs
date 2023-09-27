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

#![allow(unused_imports)]

use std::ffi::OsString;
use std::path::PathBuf;

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;

/// Dredge is a command line tool for working with the Docker Registry
/// V2 API.
#[derive(Debug, Parser, PartialEq, Eq)]
#[command(name = "dredge", version, author)]
#[command(about, long_about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Commands,

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

    /// The host or host:port or full base URL of the Docker Registry
    pub registry: String,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
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

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum Commands {
    /// Fetch the list of available repositories from the catalog.
    Catalog,

    /// Fetch the list of tags for a given image.
    #[command(arg_required_else_help = true)]
    Tags { name: String },

    /// Show detailed information about a particular image.
    #[command(arg_required_else_help = true)]
    Show {
        image: String,
        #[arg(default_missing_value = "latest")]
        tag: Option<String>,
    },

    /// Delete a tagged image from the registry.
    #[command(arg_required_else_help = true)]
    Delete { image: String, tag: String },

    /// Perform a simple version check towards the Docker Registry API
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
