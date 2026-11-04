use chrono::format::StrftimeItems;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::config::kdl_config::{parse_kdl, KdlConfigError};
use crate::config::toml_config::parse_toml;
use crate::config::toml_config::TomlConfigError;
use crate::config::Config;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum ConfigError {
    #[error(
        "Configuration file not found.\n\
        Make a copy of default config and either specify it as an arg or \n\
        place it in a default location.  See ReadMe for details."
    )]
    ConfigNotFound,

    #[error("The configuration file must be .kdl or .toml. Found {0}.")]
    ConfigFormatError(PathBuf),

    #[error("Invalid time format {0:?}.")]
    TimeFormatError(String),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    ConfigParseError(#[from] toml::de::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    KdlError(#[from] KdlConfigError),

    #[error(transparent)]
    TomlError(#[from] TomlConfigError),
}

fn get_config_path(config_path: Option<String>) -> Result<PathBuf, ConfigError> {
    if let Some(file_path) = config_path {
        return Ok(PathBuf::from(file_path));
    }

    let config_bases = [
        env::var("XDG_CONFIG_HOME").map(PathBuf::from),
        env::var("HOME").map(|home| Path::new(&home).join(".config")),
    ];

    for config_base in config_bases.into_iter().flatten() {
        if !config_base.is_absolute() {
            continue;
        }
        for basename in ["rust-motd/config.kdl", "rust-motd/config.toml"] {
            let path = config_base.join(basename);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    Err(ConfigError::ConfigNotFound)
}

pub fn get_config(config_path: Option<String>) -> Result<Config, ConfigError> {
    let config_path = get_config_path(config_path)?;
    let config_str = fs::read_to_string(&config_path)?;

    let extension = config_path
        .extension()
        .and_then(|extension| extension.to_str());

    let config = match extension {
        Some("toml") => parse_toml(&config_str)?,
        Some("kdl") => parse_kdl(&config_path, &config_str)?,
        _ => return Err(ConfigError::ConfigFormatError(config_path)),
    };

    let time_format = &config.global.time_format;
    if StrftimeItems::new(time_format).parse().is_err() {
        return Err(ConfigError::TimeFormatError(time_format.to_string()));
    }

    Ok(config)
}
