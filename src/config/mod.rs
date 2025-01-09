use serde::de::{Deserialize, Visitor};

pub mod get_config;
pub mod global_config;

use crate::component::BoxedComponent;
use crate::components::banner::Banner;
use crate::components::cg_stats::CgStats;
use crate::components::docker::Docker;
use crate::components::fail_2_ban::Fail2Ban;
use crate::components::filesystem::Filesystems;
use crate::components::last_login::LastLogin;
use crate::components::last_run::LastRun;
use crate::components::loadavg::LoadAvg;
use crate::components::memory::Memory;
use crate::components::service_status::{ServiceStatus, UserServiceStatus};
use crate::components::ssl_certs::SSLCerts;
use crate::components::uptime::Uptime;
use crate::components::weather::Weather;
use global_config::GlobalConfig;

/// The fields available in the config file
/// This includes all components plus the global configuration settings
#[derive(Debug, serde::Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum Fields {
    Global,
    Banner,
    CgStats,
    Docker,
    #[serde(rename = "fail_2_ban")]
    Fail2Ban,
    Filesystems,
    LastLogin,
    LastRun,
    LoadAvg,
    Memory,
    ServiceStatus,
    UserServiceStatus,
    #[serde(rename = "ssl_certificates")]
    SSLCerts,
    Uptime,
    Weather,
}

/// Configuration for all components and the global settings
/// The order of the components in the vector is the order they appear in the configuration file
/// and is the order in which they should be printed
/// This way, users can configure the order of components by shifting lines in the config file
pub struct Config {
    pub components: Vec<BoxedComponent>,
    pub global: GlobalConfig,
}

// Deserializer that pushes components in the order they appear in the configuration file
// Reference: https://serde.rs/deserialize-struct.html
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
                                .push(Box::new(map.next_value::<Banner>()?));
                        }
                        Fields::CgStats => {
                            result
                                .components
                                .push(Box::new(map.next_value::<CgStats>()?));
                        }
                        Fields::Docker => {
                            result.components.push(Box::new(Docker {
                                containers: map.next_value()?,
                            }));
                        }
                        Fields::Fail2Ban => {
                            result
                                .components
                                .push(Box::new(map.next_value::<Fail2Ban>()?));
                        }
                        Fields::Filesystems => {
                            result
                                .components
                                .push(Box::new(Filesystems::new(map.next_value()?)));
                        }
                        Fields::LastLogin => {
                            result.components.push(Box::new(LastLogin {
                                users: map.next_value()?,
                            }));
                        }
                        Fields::LastRun => {
                            result
                                .components
                                .push(Box::new(map.next_value::<LastRun>()?));
                        }
                        Fields::LoadAvg => {
                            result
                                .components
                                .push(Box::new(map.next_value::<LoadAvg>()?));
                        }
                        Fields::Memory => {
                            result
                                .components
                                .push(Box::new(map.next_value::<Memory>()?));
                        }
                        Fields::ServiceStatus => {
                            result.components.push(Box::new(ServiceStatus {
                                services: map.next_value()?,
                            }));
                        }
                        Fields::UserServiceStatus => {
                            result.components.push(Box::new(UserServiceStatus {
                                services: map.next_value()?,
                            }));
                        }
                        Fields::SSLCerts => {
                            result
                                .components
                                .push(Box::new(map.next_value::<SSLCerts>()?));
                        }
                        Fields::Uptime => {
                            result
                                .components
                                .push(Box::new(map.next_value::<Uptime>()?));
                        }
                        Fields::Weather => {
                            result
                                .components
                                .push(Box::new(map.next_value::<Weather>()?));
                        }
                    }
                }
                Ok(result)
            }
        }

        deserializer.deserialize_map(ConfigVisitor)
    }
}
