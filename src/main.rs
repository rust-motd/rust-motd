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
use components::last_run::{disp_last_run, LastRunConfig};
use components::service_status::{disp_service_status, ServiceStatusCfg};
use components::ssl_certs::{disp_ssl, SSLCertsCfg};
use components::uptime::{disp_uptime, UptimeCfg};
use components::weather::{disp_weather, WeatherCfg};
mod command;
mod constants;
use constants::GlobalSettings;

#[derive(Debug, Deserialize)]
struct Config {
    banner: Option<BannerCfg>,
    service_status: Option<ServiceStatusCfg>,
    user_service_status: Option<ServiceStatusCfg>,
    uptime: Option<UptimeCfg>,
    ssl_certificates: Option<SSLCertsCfg>,
    filesystems: Option<FilesystemsCfg>,
    fail_2_ban: Option<Fail2BanCfg>,
    last_login: Option<LastLoginCfg>,
    weather: Option<WeatherCfg>,
    last_run: Option<LastRunConfig>,
    #[serde(default)]
    global: GlobalSettings,
}

fn main() {
    let args = env::args();
    match get_config(args) {
        Ok(config) => {
            let sys = System::new();

            if let Some(banner_config) = config.banner {
                disp_banner(banner_config).unwrap_or_else(|err| println!("Banner error: {}", err));
                println!();
            }

            if let Some(weather_config) = config.weather {
                disp_weather(weather_config)
                    .unwrap_or_else(|err| println!("Weather error: {}", err));
                println!();
            }

            if let Some(uptime_config) = config.uptime {
                disp_uptime(uptime_config, &sys)
                    .unwrap_or_else(|err| println!("Uptime error: {}", err));
                println!();
            }

            if let Some(service_status_config) = config.service_status {
                println!("System Services:");
                disp_service_status(service_status_config, false)
                    .unwrap_or_else(|err| println!("Service status error: {}", err));
                println!();
            }

            if let Some(service_status_config) = config.user_service_status {
                println!("User Services:");
                disp_service_status(service_status_config, true)
                    .unwrap_or_else(|err| println!("User service status error: {}", err));
                println!();
            }

            if let Some(ssl_certificates_config) = config.ssl_certificates {
                disp_ssl(ssl_certificates_config, &config.global)
                    .unwrap_or_else(|err| println!("SSL Certificate error: {}", err));
                println!();
            }

            if let Some(filesystems) = config.filesystems {
                disp_filesystem(filesystems, &config.global, &sys)
                    .unwrap_or_else(|err| println!("Filesystem error: {}", err));
                println!();
            }

            if let Some(last_login_config) = config.last_login {
                disp_last_login(last_login_config, &config.global)
                    .unwrap_or_else(|err| println!("Last login error: {}", err));
                println!();
            }

            if let Some(fail_2_ban_config) = config.fail_2_ban {
                disp_fail_2_ban(fail_2_ban_config)
                    .unwrap_or_else(|err| println!("Fail2Ban error: {}", err));
                println!();
            }

            if let Some(last_run_config) = config.last_run {
                disp_last_run(last_run_config, &config.global)
                    .unwrap_or_else(|err| println!("Last run error: {}", err));
            }
        }
        Err(e) => println!("Config Error: {}", e),
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(
        "Configuration file not found.\n\
        Make a copy of default config and either specify it as an arg or \n\
        place it in a default location.  See ReadMe for details."
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
            let config_base = Path::new(&config_base).join(Path::new("rust-motd/config.toml"));
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
