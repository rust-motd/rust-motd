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
    ConfigFormatError(String),

    #[error(transparent)]
    ConfigHomeError(#[from] std::env::VarError),

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

    let config_base = env::var("XDG_CONFIG_HOME").unwrap_or(env::var("HOME")? + "/.config");

    for basename in ["rust-motd/config.kdl", "rust-motd/config.toml"] {
        let config_base = Path::new(&config_base).join(Path::new(basename));
        if config_base.exists() {
            return Ok(config_base);
        }
    }

    Err(ConfigError::ConfigNotFound)
}

pub fn get_config(config_path: Option<String>) -> Result<Config, ConfigError> {
    let config_path = get_config_path(config_path)?;
    let config_str = fs::read_to_string(&config_path)?;

    let extension = config_path
        .extension()
        .expect("Could not determine extension for config file.");
    let extension = extension
        .to_str()
        .expect("Could not determine extension for config file.");

    match extension {
        "toml" => Ok(parse_toml(&config_str)?),
        "kdl" => Ok(parse_kdl(&config_path, &config_str)?),
        other => Err(ConfigError::ConfigFormatError(other.to_string())),
    }
}
