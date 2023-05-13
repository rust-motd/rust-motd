use crate::constants::INDENT_WIDTH;
use async_trait::async_trait;
use docker_api::models::ContainerSummary;
use docker_api::opts::ContainerListOpts;
use docker_api::{Docker as DockerAPI, Result as DockerResult};
use std::collections::HashMap;
use termion::{color, style};

use crate::component::Component;
use crate::config::global_config::GlobalConfig;
use crate::default_prepare;

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
    default_prepare!();
}

#[cfg(unix)]
pub fn new_docker() -> DockerResult<DockerAPI> {
    Ok(DockerAPI::unix("/var/run/docker.sock"))
}

#[cfg(not(unix))]
pub fn new_docker() -> DockerResult<DockerAPI> {
    DockerAPI::new("tcp://127.0.0.1:8080")
}

struct Container {
    summary: ContainerSummary,
    name: String,
}

impl Docker {
    pub async fn print_or_error(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let docker = new_docker()?;

        // Get all containers from library and then filter them
        // Not perfect, but I got strange issues when trying to use `.get(id)`
        let containers: Vec<Container> = docker
            .containers()
            .list(&ContainerListOpts::builder().all(true).build())
            .await?
            .into_iter()
            .filter_map(|container| {
                match container.names.as_ref() {
                    Some(names) => names.iter().find_map(|name| {
                        self.containers
                            .remove_entry(name)
                            .map(|(_docker_name, display_name)| Container {
                                name: display_name,
                                summary: container.clone(),
                            })
                    }),
                    _ => None,
                }
                // container.names.as_ref().map(|names| {
                //     names.iter().find_map(|name| {
                //         self.containers.remove_entry(name).map(|(_docker_name, display_name)| (display_name, container))
                //     })
                // })
            })
            .collect();

        for (docker_name, _display_name) in self.containers {
            println!(
                "{indent}{color}Warning: Could not find Docker container `{docker_name}'{reset}",
                indent = " ".repeat(INDENT_WIDTH),
                color = color::Fg(color::Yellow),
                docker_name = docker_name,
                reset = style::Reset
            );
        }

        // Max length of all the container names (first column)
        // to determine the padding
        if let Some(max_len) = containers
            .iter()
            .map(|container| container.name.len())
            .max()
        {
            for container in containers {
                let status_color = match container.summary.state.as_deref() {
                    Some("Created") | Some("Restarting") | Some("Paused") | Some("Removing")
                    | Some("Configured") => color::Fg(color::Yellow).to_string(),
                    Some("Running") => color::Fg(color::Green).to_string(),
                    Some("Exited") => color::Fg(color::LightBlack).to_string(),
                    Some("Dead") => color::Fg(color::Red).to_string(),
                    _ => color::Fg(color::White).to_string(),
                };
                println!(
                    "{indent}{name}: {padding}{color}{status}{reset}",
                    indent = " ".repeat(INDENT_WIDTH),
                    name = container.name,
                    padding = " ".repeat(max_len - container.name.len()),
                    color = status_color,
                    status = container.summary.status.unwrap_or(String::from("?")),
                    reset = style::Reset,
                );
            }
        }

        Ok(())
    }
}
