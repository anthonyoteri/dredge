#![allow(unused_imports)]

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use std::ffi::OsString;
use std::path::PathBuf;

/// Dredge is a command line tool for working with the Docker Registry
/// V2 API.
#[derive(Debug, Parser)]
#[command(name = "dredge", version, author)]
#[command(about, long_about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Optional configuration file override.
    #[arg(short = 'c', long = "config")]
    pub config: Option<OsString>,

    #[arg(
        long = "log-level",
        require_equals = true,
        value_name = "LEVEL",
        num_args = 0..=1,
        default_value_t = LogLevel::Info,
        default_missing_value="info",
        value_enum
    )]
    pub log_level: LogLevel,
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

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Fetch the list of available repositories from the catalog.
    Catalog,

    /// Perform a simple API Version check towards the configured registry
    /// endpoint.
    Check,
}
