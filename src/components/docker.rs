use crate::constants::INDENT_WIDTH;
use async_trait::async_trait;
use docker_api::api::container::ContainerStatus;
use docker_api::container::ContainerListOpts;
use docker_api::{Docker as DockerAPI, Result as DockerResult};
use std::collections::HashMap;
use termion::{color, style};

use crate::component::{Component, PrepareReturn};
use crate::config::global_config::GlobalConfig;

pub struct Docker {
    pub containers: HashMap<String, String>,
}

#[async_trait]
impl Component for Docker {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        println!("Docker:");
        self.print_or_error()
            .await
            .unwrap_or_else(|err| println!("Docker status error: {}", err));
        println!();
    }
    fn prepare(self: Box<Self>, _global_config: &GlobalConfig) -> PrepareReturn {
        (self, None)
    }
}

#[cfg(unix)]
pub fn new_docker() -> DockerResult<DockerAPI> {
    Ok(DockerAPI::unix("/var/run/docker.sock"))
}

#[cfg(not(unix))]
pub fn new_docker() -> DockerResult<DockerAPI> {
    DockerAPI::new("tcp://127.0.0.1:8080")
}

impl Docker {
    pub async fn print_or_error(self) -> Result<(), Box<dyn std::error::Error>> {
        let docker = new_docker()?;

        // Get all containers from library and then filter them
        // Not perfect, but I got strange issues when trying to use `.get(id)`
        let containers = docker
            .containers()
            .list(&ContainerListOpts::builder().all(true).build())
            .await?;
        let containers = self
            .containers
            .iter()
            .filter_map(|(docker_name, display_name)| {
                match containers
                    .iter()
                    .find(|container| container.names.contains(docker_name))
                {
                    Some(container) => Some((display_name, container)),
                    None => {
                        println!(
                            "{indent}{color}Could not find container with name `{name}'{reset}",
                            indent = " ".repeat(INDENT_WIDTH as usize),
                            color = color::Fg(color::Yellow),
                            name = docker_name,
                            reset = style::Reset
                        );
                        None
                    }
                }
            })
            .collect::<Vec<_>>();

        // Max length of all the container names (first column)
        // to determine the padding
        if let Some(max_len) = self.containers.values().map(|v| v.len()).max() {
            for (name, container) in containers {
                let status_color = match container.state {
                    ContainerStatus::Created
                    | ContainerStatus::Restarting
                    | ContainerStatus::Paused
                    | ContainerStatus::Removing
                    | ContainerStatus::Configured => color::Fg(color::Yellow).to_string(),
                    ContainerStatus::Running => color::Fg(color::Green).to_string(),
                    ContainerStatus::Exited => color::Fg(color::LightBlack).to_string(),
                    ContainerStatus::Dead => color::Fg(color::Red).to_string(),
                };
                println!(
                    "{indent}{name}: {padding}{color}{status}{reset}",
                    indent = " ".repeat(INDENT_WIDTH as usize),
                    name = name,
                    padding = " ".repeat(max_len - name.len()),
                    color = status_color,
                    status = container.status,
                    reset = style::Reset,
                );
            }
        }

        Ok(())
    }
}
