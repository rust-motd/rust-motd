use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use systemstat::{Platform, System};
use thiserror::Error;

mod components;
use components::banner::{disp_banner, BannerCfg};
use components::fail_2_ban::{disp_fail_2_ban, Fail2BanCfg};
use components::filesystem::{disp_filesystem, FilesystemsCfg};
use components::last_login::{disp_last_login, LastLoginCfg};
use components::service_status::{disp_service_status, ServiceStatusCfg};
use components::ssl_certs::{disp_ssl, SSLCertsCfg};
use components::uptime::{disp_uptime, UptimeCfg};
mod constants;

#[derive(Debug, Deserialize)]
struct Config {
    banner: Option<BannerCfg>,
    service_status: Option<ServiceStatusCfg>,
    uptime: Option<UptimeCfg>,
    ssl_certificates: Option<SSLCertsCfg>,
    filesystems: Option<FilesystemsCfg>,
    fail_2_ban: Option<Fail2BanCfg>,
    last_login: Option<LastLoginCfg>,
}

fn main() {
    let args = env::args();
    match get_config(args) {
        Ok(config) => {
            let sys = System::new();

            if let Some(banner_config) = config.banner {
                disp_banner(banner_config).unwrap_or_else(|err| println!("Banner error: {}", err));
            }

            if let Some(uptime_config) = config.uptime {
                disp_uptime(uptime_config, &sys)
                    .unwrap_or_else(|err| println!("Uptime error: {}", err));
            }

            if let Some(service_status_config) = config.service_status {
                disp_service_status(service_status_config)
                    .unwrap_or_else(|err| println!("Service status error: {}", err));
            }

            if let Some(ssl_certificates_config) = config.ssl_certificates {
                disp_ssl(ssl_certificates_config)
                    .unwrap_or_else(|err| println!("SSL Certificate error: {}", err));
            }

            if let Some(filesystems) = config.filesystems {
                disp_filesystem(filesystems, &sys)
                    .unwrap_or_else(|err| println!("Filesystem error: {}", err));
            }

            if let Some(last_login_config) = config.last_login {
                disp_last_login(last_login_config)
                    .unwrap_or_else(|err| println!("Last login error: {}", err));
            }

            if let Some(fail_2_ban_config) = config.fail_2_ban {
                disp_fail_2_ban(fail_2_ban_config)
                    .unwrap_or_else(|err| println!("Fail2Ban error: {}", err));
            }
        }
        Err(e) => println!("Config Error: {}", e),
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(
        "Configuration file not found. Config file needs to be passed either as \
        an arg or in a default location. See ReadMe for details."
    )]
    ConfigNotFound,

    #[error(transparent)]
    ConfigHomeError(#[from] std::env::VarError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    ConfigParseError(#[from] toml::de::Error),
}

fn get_config(mut args: env::Args) -> Result<Config, ConfigError> {
    let config_path = match args.nth(1) {
        Some(file_path) => Some(PathBuf::from(file_path)),
        None => {
            let config_base = env::var("XDG_CONFIG_HOME").unwrap_or(env::var("HOME")? + "/.config");
            let config_base = Path::new(&config_base).join(Path::new("motd-rust/config.toml"));
            if config_base.exists() {
                Some(config_base)
            } else {
                None
            }
        }
    };
    match config_path {
        Some(path) => Ok(toml::from_str(&fs::read_to_string(path)?)?),
        None => Err(ConfigError::ConfigNotFound),
    }
}
