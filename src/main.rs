use serde::de::{Deserialize, Visitor};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use systemstat::{Platform, System};
use thiserror::Error;

mod components;
use components::banner::{disp_banner, BannerCfg};
use components::docker::{disp_docker, DockerConfig};
use components::fail_2_ban::{disp_fail_2_ban, Fail2BanCfg};
use components::filesystem::{disp_filesystem, FilesystemsCfg};
use components::last_login::{disp_last_login, LastLoginCfg};
use components::last_run::{disp_last_run, LastRunConfig};
use components::memory::{disp_memory, MemoryCfg};
use components::service_status::{disp_service_status, ServiceStatusCfg};
use components::ssl_certs::{disp_ssl, SSLCertsCfg};
use components::uptime::{disp_uptime, UptimeCfg};
use components::weather::{disp_weather, WeatherCfg};
mod command;
mod constants;
use constants::GlobalSettings;

#[derive(Debug, serde::Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum Fields {
    Global,
    Banner,
    Docker,
    Fail2Ban,
    Filesystems,
    LastLogin,
    LastRun,
    Memory,
    ServiceStatus,
    UserServiceStatus,
    SSLCerts,
    Uptime,
    Weather,
}

#[derive(Debug)]
// #[derive(Debug, EnumDiscriminants)]
// #[strum_discriminants(derive(EnumString, EnumMessage, serde::Deserialize))]
// #[strum_discriminants(serde(field_identifier, rename_all = "snake_case"))]
enum ComponentConfig {
    Banner(BannerCfg),
    Docker(DockerConfig),
    Fail2Ban(Fail2BanCfg),
    Filesystems(FilesystemsCfg),
    LastLogin(LastLoginCfg),
    LastRun(LastRunConfig),
    Memory(MemoryCfg),
    ServiceStatus(ServiceStatusCfg),
    UserServiceStatus(ServiceStatusCfg),
    SSLCerts(SSLCertsCfg),
    Uptime(UptimeCfg),
    Weather(WeatherCfg),
}

#[derive(Debug)]
struct Config {
    components: Vec<ComponentConfig>,
    global: GlobalSettings,
}

// https://serde.rs/deserialize-struct.html
impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ConfigVisitor;

        impl<'de> Visitor<'de> for ConfigVisitor {
            type Value = Config;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Config")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut result = Config {
                    components: vec![],
                    global: GlobalSettings::default(),
                };

                while let Some(key) = map.next_key()? {
                    match key {
                        Fields::Global => {
                            result.global = map.next_value()?;
                        }
                        Fields::Banner => {
                            result
                                .components
                                .push(ComponentConfig::Banner(map.next_value()?));
                        }
                        Fields::Docker => {
                            result
                                .components
                                .push(ComponentConfig::Docker(map.next_value()?));
                        }
                        Fields::Fail2Ban => {
                            result
                                .components
                                .push(ComponentConfig::Fail2Ban(map.next_value()?));
                        }
                        Fields::Filesystems => {
                            result
                                .components
                                .push(ComponentConfig::Filesystems(map.next_value()?));
                        }
                        Fields::LastLogin => {
                            result
                                .components
                                .push(ComponentConfig::LastLogin(map.next_value()?));
                        }
                        Fields::LastRun => {
                            result
                                .components
                                .push(ComponentConfig::LastRun(map.next_value()?));
                        }
                        Fields::Memory => {
                            result
                                .components
                                .push(ComponentConfig::Memory(map.next_value()?));
                        }
                        Fields::ServiceStatus => {
                            result
                                .components
                                .push(ComponentConfig::ServiceStatus(map.next_value()?));
                        }
                        Fields::UserServiceStatus => {
                            result
                                .components
                                .push(ComponentConfig::UserServiceStatus(map.next_value()?));
                        }
                        Fields::SSLCerts => {
                            result
                                .components
                                .push(ComponentConfig::SSLCerts(map.next_value()?));
                        }
                        Fields::Uptime => {
                            result
                                .components
                                .push(ComponentConfig::Uptime(map.next_value()?));
                        }
                        Fields::Weather => {
                            result
                                .components
                                .push(ComponentConfig::Weather(map.next_value()?));
                        }
                    }
                }
                Ok(result)
            }
        }

        deserializer.deserialize_map(ConfigVisitor)
    }
}

async fn print_motd(config: Config) {
    let sys = System::new();
    let mut bar_size_hint: Option<usize> = None;

    for component_config in config.components {
        match component_config {
            ComponentConfig::Banner(banner_config) => {
                disp_banner(banner_config).unwrap_or_else(|err| println!("Banner error: {}", err));
                println!();
            }
            ComponentConfig::Docker(docker_config) => {
                println!("Docker:");
                disp_docker(docker_config)
                    .await
                    .unwrap_or_else(|err| println!("Docker status error: {}", err));
                println!();
            }
            ComponentConfig::Fail2Ban(fail_2_ban_config) => {
                disp_fail_2_ban(fail_2_ban_config)
                    .unwrap_or_else(|err| println!("Fail2Ban error: {}", err));
                println!();
            }
            ComponentConfig::Filesystems(filesystems_config) => {
                bar_size_hint = disp_filesystem(filesystems_config, &config.global, &sys)
                    .unwrap_or_else(|err| {
                        println!("Filesystem error: {}", err);
                        None
                    });
                println!();
            }
            ComponentConfig::LastLogin(last_login_config) => {
                disp_last_login(last_login_config, &config.global)
                    .unwrap_or_else(|err| println!("Last login error: {}", err));
                println!();
            }
            ComponentConfig::LastRun(last_run_config) => {
                disp_last_run(last_run_config, &config.global)
                    .unwrap_or_else(|err| println!("Last run error: {}", err));
            }
            ComponentConfig::Memory(memory_config) => {
                disp_memory(memory_config, &config.global, &sys, bar_size_hint) // TODO:
                    .unwrap_or_else(|err| println!("Memory error: {}", err));
                println!();
            }
            ComponentConfig::ServiceStatus(service_status_config) => {
                println!("System Services:");
                disp_service_status(service_status_config, false)
                    .unwrap_or_else(|err| println!("Service status error: {}", err));
                println!();
            }
            ComponentConfig::UserServiceStatus(user_service_status_config) => {
                println!("User Services:");
                disp_service_status(user_service_status_config, true)
                    .unwrap_or_else(|err| println!("User service status error: {}", err));
                println!();
            }
            ComponentConfig::SSLCerts(ssl_certificates_config) => {
                disp_ssl(ssl_certificates_config, &config.global)
                    .unwrap_or_else(|err| println!("SSL Certificate error: {}", err));
                println!();
            }
            ComponentConfig::Uptime(uptime_config) => {
                disp_uptime(uptime_config, &sys)
                    .unwrap_or_else(|err| println!("Uptime error: {}", err));
                println!();
            }
            ComponentConfig::Weather(weather_config) => {
                disp_weather(weather_config)
                    .unwrap_or_else(|err| println!("Weather error: {}", err));
                println!();
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args();

    match get_config(args) {
        Ok(config) => {
            print_motd(config).await;
        }
        Err(e) => println!("Config Error: {}", e),
    }
    Ok(())
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
