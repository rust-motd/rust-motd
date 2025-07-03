use std::path::Path;
use thiserror::Error;

use crate::component::BoxedComponent;
use crate::components::cg_stats::CgStats;
use crate::components::command::Command;
use crate::components::docker::Docker;
use crate::components::docker_compose::DockerCompose;
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
use crate::config::global_config::GlobalConfig;
use crate::config::Config;

const EXPECTED_VERSION: &str = "1.0";

#[derive(knus::Decode, Debug)]
pub enum ComponentNode {
    Command(Command),
    CgStats(CgStats),
    Docker(Docker),
    DockerCompose(DockerCompose),
    Fail2ban(Fail2Ban),
    Filesystems(Filesystems),
    LastLogin(LastLogin),
    LastRun(LastRun),
    LoadAvg(LoadAvg),
    Memory(Memory),
    SSLCerts(SSLCerts),
    ServiceStatus(ServiceStatus),
    Uptime(Uptime),
    UserServiceStatus(UserServiceStatus),
    Weather(Weather),
}

#[derive(knus::Decode, Debug)]
pub struct KdlConfig {
    #[knus(child)]
    pub global: GlobalConfig,
    #[knus(child, unwrap(children))]
    pub components: Vec<ComponentNode>,
}

#[derive(Error, Debug, miette::Diagnostic)]
pub enum KdlConfigError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    KnusError(#[from] knus::Error),

    #[error("Mandatory field `global {{ version \"{0}\" }}` is missing in the configuration.")]
    NoVersion(&'static str),

    #[error("The only supported config version is {0} but your config has version {1}.")]
    VersionMismatch(&'static str, String),
}

pub fn parse_kdl(config_path: &Path, config_str: &str) -> Result<Config, KdlConfigError> {
    let result = knus::parse::<KdlConfig>(config_path.to_str().unwrap(), config_str)?;

    let version = result
        .global
        .version
        .clone()
        .ok_or(KdlConfigError::NoVersion(EXPECTED_VERSION))?;

    if EXPECTED_VERSION != version {
        return Err(KdlConfigError::VersionMismatch(EXPECTED_VERSION, version));
    }

    Ok(Config {
        global: result.global,
        components: result
            .components
            .into_iter()
            .map(|x| match x {
                ComponentNode::Command(command) => Box::new(command) as BoxedComponent,
                ComponentNode::CgStats(stats) => Box::new(stats) as BoxedComponent,
                ComponentNode::Docker(docker) => Box::new(docker) as BoxedComponent,
                ComponentNode::DockerCompose(compose) => Box::new(compose) as BoxedComponent,
                ComponentNode::Fail2ban(fail2ban) => Box::new(fail2ban) as BoxedComponent,
                ComponentNode::Filesystems(filesystems) => Box::new(filesystems) as BoxedComponent,
                ComponentNode::LastLogin(last_login) => Box::new(last_login) as BoxedComponent,
                ComponentNode::LastRun(last_run) => Box::new(last_run) as BoxedComponent,
                ComponentNode::LoadAvg(load_avg) => Box::new(load_avg) as BoxedComponent,
                ComponentNode::Memory(memory) => Box::new(memory) as BoxedComponent,
                ComponentNode::SSLCerts(certs) => Box::new(certs) as BoxedComponent,
                ComponentNode::ServiceStatus(service_status) => {
                    Box::new(service_status) as BoxedComponent
                }
                ComponentNode::UserServiceStatus(user_service_status) => {
                    Box::new(user_service_status) as BoxedComponent
                }
                ComponentNode::Uptime(uptime) => Box::new(uptime) as BoxedComponent,
                ComponentNode::Weather(weather) => Box::new(weather) as BoxedComponent,
            })
            .collect(),
    })
}
