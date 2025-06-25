use indexmap::IndexMap;
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use termion::{color, style};
use thiserror::Error;

use crate::components::cg_stats::CgStats;
use crate::components::command::Command;
use crate::components::docker::{Docker, DockerContainer};
use crate::components::fail_2_ban::Fail2Ban;
use crate::components::filesystem::{Filesystems, Mount};
use crate::components::last_login::{LastLogin, User};
use crate::components::last_run::LastRun;
use crate::components::loadavg::LoadAvg;
use crate::components::memory::Memory;
use crate::components::service_status::{Service, ServiceStatus, UserServiceStatus};
use crate::components::ssl_certs::SSLCerts;
use crate::components::uptime::Uptime;
use crate::components::weather::Weather;
use crate::config::global_config::GlobalConfig;
use crate::config::Config;

/// The fields available in the config file
/// This includes all components plus the global configuration settings
#[derive(Debug, serde::Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum Fields {
    Global,
    // Command component was called banner in the legacy configuration format.
    #[serde(rename = "banner")]
    Command,
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

#[derive(Error, Debug)]
pub enum TomlConfigError {
    #[error(transparent)]
    ConfigParseError(#[from] toml::de::Error),
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
                        Fields::Command => {
                            result
                                .components
                                .push(Box::new(map.next_value::<Command>()?));
                        }
                        Fields::CgStats => {
                            result
                                .components
                                .push(Box::new(map.next_value::<CgStats>()?));
                        }
                        Fields::Docker => {
                            result.components.push(Box::new(Docker {
                                containers: map
                                    .next_value::<IndexMap<String, String>>()?
                                    .into_iter()
                                    .map(|(docker_name, display_name)| DockerContainer {
                                        docker_name,
                                        display_name,
                                    })
                                    .collect(),
                            }));
                        }
                        Fields::Fail2Ban => {
                            result
                                .components
                                .push(Box::new(map.next_value::<Fail2Ban>()?));
                        }
                        Fields::Filesystems => {
                            result.components.push(Box::new(Filesystems::new(
                                map.next_value::<IndexMap<String, String>>()?
                                    .into_iter()
                                    .map(|(name, mount_point)| Mount { name, mount_point })
                                    .collect(),
                            )));
                        }
                        Fields::LastLogin => {
                            result.components.push(Box::new(LastLogin {
                                users: map
                                    .next_value::<IndexMap<String, usize>>()?
                                    .into_iter()
                                    .map(|(username, num_logins)| User {
                                        username,
                                        num_logins,
                                    })
                                    .collect(),
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
                                services: map
                                    .next_value::<IndexMap<String, String>>()?
                                    .into_iter()
                                    .map(|(display_name, unit)| Service { display_name, unit })
                                    .collect(),
                            }));
                        }
                        Fields::UserServiceStatus => {
                            result.components.push(Box::new(UserServiceStatus {
                                services: map
                                    .next_value::<IndexMap<String, String>>()?
                                    .into_iter()
                                    .map(|(display_name, unit)| Service { display_name, unit })
                                    .collect(),
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

pub fn deserialize_certs<'de, D>(
    deserializer: D,
) -> Result<Vec<crate::components::ssl_certs::Cert>, D::Error>
where
    D: Deserializer<'de>,
{
    struct CertsVisitor;

    impl<'de> Visitor<'de> for CertsVisitor {
        type Value = Vec<crate::components::ssl_certs::Cert>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("list of certificates")
        }

        fn visit_map<M>(
            self,
            mut map: M,
        ) -> Result<Vec<crate::components::ssl_certs::Cert>, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut result = vec![];
            while let Some((name, path)) = map.next_entry::<_, String>()? {
                result.push(crate::components::ssl_certs::Cert { name, path });
            }
            Ok(result)
        }
    }

    deserializer.deserialize_map(CertsVisitor)
}

pub fn parse_toml(config_str: &str) -> Result<Config, TomlConfigError> {
    let config: Config = toml::from_str(config_str)?;

    if config.global.show_legacy_warning {
        println!(
            concat!(
            "{}You are using the legacy TOML configuration format.\n",
            "Support may be removed in the next major release.\n",
            "Please upgrade to the KDL configuration format ",
            "(see https://github.com/rust-motd/rust-motd/tree/main/docs/config_migration.md).\n",
            "Add `show_legacy_warning = false` to the `[global]` section ",
            "of your TOML config to silence this.{}",
        ),
            color::Yellow.fg_str(),
            style::Reset
        );
    }

    Ok(config)
}
