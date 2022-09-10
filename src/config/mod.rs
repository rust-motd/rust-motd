use serde::de::{Deserialize, Visitor};

pub mod global_config;
pub mod get_config;

use crate::components::banner::BannerConfig;
use crate::components::docker::DockerConfig;
use crate::components::fail_2_ban::Fail2BanConfig;
use crate::components::filesystem::FilesystemsConfig;
use crate::components::last_login::LastLoginConfig;
use crate::components::last_run::LastRunConfig;
use crate::components::memory::MemoryConfig;
use crate::components::service_status::ServiceStatusConfig;
use crate::components::ssl_certs::SSLCertsConfig;
use crate::components::uptime::UptimeConfig;
use crate::components::weather::WeatherConfig;
use global_config::GlobalConfig;

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
pub enum ComponentConfig {
    Banner(BannerConfig),
    Docker(DockerConfig),
    Fail2Ban(Fail2BanConfig),
    Filesystems(FilesystemsConfig),
    LastLogin(LastLoginConfig),
    LastRun(LastRunConfig),
    Memory(MemoryConfig),
    ServiceStatus(ServiceStatusConfig),
    UserServiceStatus(ServiceStatusConfig),
    SSLCerts(SSLCertsConfig),
    Uptime(UptimeConfig),
    Weather(WeatherConfig),
}

#[derive(Debug)]
pub struct Config {
    pub components: Vec<ComponentConfig>,
    pub global: GlobalConfig,
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
                    global: GlobalConfig::default(),
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
