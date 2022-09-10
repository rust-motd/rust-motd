use serde::de::{Deserialize, Visitor};

pub mod get_config;
pub mod global_config;

use crate::component::Component;
use crate::components::banner::Banner;
use crate::components::docker::Docker;
use crate::components::fail_2_ban::Fail2Ban;
use crate::components::filesystem::Filesystems;
use crate::components::last_login::LastLogin;
use crate::components::last_run::LastRun;
use crate::components::memory::Memory;
use crate::components::service_status::{ServiceStatus, UserServiceStatus};
use crate::components::ssl_certs::SSLCerts;
use crate::components::uptime::Uptime;
use crate::components::weather::Weather;
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

pub struct Config {
    pub components: Vec<Box<dyn Component>>,
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
                                .push(Box::new(map.next_value::<Banner>()?));
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
                            result.components.push(Box::new(Filesystems {
                                mounts: map.next_value()?,
                            }));
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
