use crate::cli::Cli;
use crate::cli::Commands;
use crate::config::Config;
use crate::error::ConfigError;
use crate::error::DredgeError;
use clap::Parser;
use std::ffi::OsString;
use std::path::PathBuf;

pub(crate) mod cli;
mod commands;
mod config;
mod error;

/// The default basename of the main configuration file.
const CONFIG_FILE_NAME: &str = "dredge.toml";

/// The XDG directory prefix.
const CONFIG_PREFIX: &str = "dredge";

/// Locate the absolute path to the saved configuration file on disk.
///
/// If given an optional `path` to a configuration file, and that file
/// exists on disk, the absoulte path to that file will be returned.
/// Otherwise, the XDG configuration path will be used.  If neither the
/// optional `path` parameter refers to an existing file on disk, nor a
/// suitable configuration file can be located within the XDG configuration
/// path, the `None` variant will be returned.
fn locate_config_file(path: Option<OsString>) -> Option<PathBuf> {
    log::trace!("locate_config_file({path:?})");

    match path {
        Some(path) => {
            let p = PathBuf::from(path);
            log::debug!("Checking if path {p:?} exists");
            p.try_exists().map(|_| Some(p)).unwrap_or(None)
        }
        None => {
            let xdg_dirs = xdg::BaseDirectories::with_prefix(CONFIG_PREFIX).ok()?;
            let search_paths: Vec<PathBuf> = vec![xdg_dirs.get_config_home()]
                .into_iter()
                .chain(xdg_dirs.get_config_dirs())
                .collect();

            log::debug!(
                "Searching configuration directories for {CONFIG_FILE_NAME} {search_paths:?}"
            );
            xdg_dirs.find_config_file(CONFIG_FILE_NAME)
        }
    }
}

/// Attempt to create a default configuration file in the XDG configuration
/// path.  Any sub-directories of the XDG configuration path which do not
/// already exist will be created automatically.
///
/// # Errors:
///
/// This returns a `ConfigError` if a problem occured which prevented either
/// the creation of the directory tree, or in writing the default configuration
/// to the file.
fn create_default_config_file() -> Result<PathBuf, ConfigError> {
    log::trace!("create_default_config_file()");

    let xdg_dirs = xdg::BaseDirectories::with_prefix(CONFIG_PREFIX)?;
    let config_path = xdg_dirs.place_config_file(CONFIG_FILE_NAME)?;
    let default_config = toml::to_string_pretty(&Config::default())?;
    std::fs::write(&config_path, default_config)?;
    Ok(config_path)
}

#[async_std::main]
async fn main() -> Result<(), DredgeError> {
    let args = Cli::parse();

    // -- Initialize logging
    let log_level = args.log_level;
    femme::with_level(log::LevelFilter::from(log_level));

    // -- Load and parse configuration file
    let config_file =
        locate_config_file(args.config).map_or_else(create_default_config_file, Ok)?;
    log::debug!("Using configuration file {config_file:?}");

    #[allow(unused_variables)]
    let config = Config::try_from(config_file.as_ref())?;

    match args.command {
        Commands::Repo(repo_args) => commands::repo::handler(&config, &repo_args).await?,
    }

    Ok(())
}
