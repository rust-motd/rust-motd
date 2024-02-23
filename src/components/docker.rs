use crate::constants::INDENT_WIDTH;
use async_trait::async_trait;
use itertools::Itertools;
use serde::Deserialize;
use docker_api::opts::{ContainerFilter, ContainerListOpts};
use docker_api::{Docker as DockerAPI, Result as DockerResult};
use std::collections::HashMap;
use termion::{color, style};

use crate::component::Component;
use crate::config::global_config::GlobalConfig;
use crate::default_prepare;

#[derive(Debug, Deserialize)]
pub struct Docker {
    containers: Option<HashMap<String, String>>,
    composes: Option<HashMap<String, String>>,
}

#[async_trait]
impl Component for Docker {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        println!("Docker:");
        let docker = match new_docker() {
            Ok(docker) => docker,
            Err(err) => {
                println!("{indent}Error: {err}", err = err, indent = " ".repeat(INDENT_WIDTH));
                return;
            }
        };

        if let Some(containers) = self.containers {
            println!("{indent}Containers:", indent = " ".repeat(INDENT_WIDTH));
            Docker::print_or_error_containers(&docker, containers)
                .await
                .unwrap_or_else(|err| println!(
                    "{indent}{red}Error getting containers: {err}{reset}",
                    indent = " ".repeat(INDENT_WIDTH),
                    red = color::Fg(color::Red),
                    reset = style::Reset,
                    err = err,
                ));
            println!();
        }

        if let Some(composes) = self.composes {
            println!("{indent}Composes:", indent = " ".repeat(INDENT_WIDTH));
            Docker::print_or_error_composes(&docker, composes)
                .await
                .unwrap_or_else(|err| println!(
                    "{indent}{red}Error getting composes: {err}{reset}",
                    indent = " ".repeat(INDENT_WIDTH),
                    red = color::Fg(color::Red),
                    reset = style::Reset,
                    err = err,
                ));
            println!();
        }
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

fn status_to_color(status: &str) -> String {
    match status {
        "running" => color::Fg(color::Green).to_string(),
        "restarting" | "created" | "configured" => color::Fg(color::Yellow).to_string(),
        "exited" | "dead" | "paused" | "removing" => color::Fg(color::LightBlack).to_string(),
        _ => color::Fg(color::White).to_string(),
    }
}

impl Docker {    
    async fn print_or_error_containers(docker: &DockerAPI, containers: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        let longest_container_name = containers.values()
            .map(|n| n.len())
            .max()
            .unwrap_or(0);

        for (container_name, name) in containers.iter() {
            let container = docker.containers()
                .list(&ContainerListOpts::builder()
                    .all(true)
                    .filter([ContainerFilter::Name(container_name.to_string())])
                    .build())
                .await?;
            
            
            match container.first() {
                Some(container) => {
                    println!(
                        "{indent}{name}: {padding}{color}{status}{reset}",
                        indent = " ".repeat(INDENT_WIDTH*2),
                        name = name,
                        padding = " ".repeat(longest_container_name - name.len()),
                        color = status_to_color(&container.state.clone().unwrap_or("exited".to_owned())),
                        status = container.status.clone().unwrap_or("unknown".to_owned()),
                        reset = style::Reset,
                    );
                },
                None => {
                    println!(
                        "{indent}{name}: {padding}{color}Not found '{container_name}'{reset}",
                        indent = " ".repeat(INDENT_WIDTH*2),
                        container_name = container_name,
                        name = name,
                        padding = " ".repeat(longest_container_name - name.len()),
                        color = color::Fg(color::Yellow),
                        reset = style::Reset,
                    );
                }
            }
        }

        Ok(())
    }

    async fn print_or_error_composes(docker: &DockerAPI, composes: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        let longest_compose_name = composes.values()
            .map(|n| n.len())
            .max()
            .unwrap_or(0);
        
        for (compose, name) in composes.iter(){ 
            let compose_containers = docker.containers()
                .list(&ContainerListOpts::builder()
                    .all(true)
                    .filter([ContainerFilter::Label("com.docker.compose.project".to_string(), compose.to_string())])
                    .build())
                .await?;

            if compose_containers.is_empty() {
                println!(
                    "{indent}{name}: {padding}{color}Not found{reset}",
                    indent = " ".repeat(INDENT_WIDTH*2),
                    name = name,
                    padding = " ".repeat(longest_compose_name - name.len()),
                    color = color::Fg(color::Yellow),
                    reset = style::Reset,
                );
                continue;
            }

            let status_grouped_containers = compose_containers.iter()
                .map(|container| {
                    (container.state.clone().unwrap_or("unknown".to_owned()), container)
                })
                .into_group_map();
            
            let mut message = format!(
                "{indent}{name}: {padding}", 
                indent = " ".repeat(INDENT_WIDTH*2),
                 name = name, 
                 padding = " ".repeat(longest_compose_name - name.len()));
            
            let running = status_grouped_containers
                .get("running").map(|v| v.len()).unwrap_or(0);
            if running > 0 {
                message.push_str(&format!(
                    "{color}Running({number}){reset} ", 
                    color = status_to_color("running"),
                    number = running,
                    reset = style::Reset,
                ));
            }

            let restarting = status_grouped_containers
                .get("restarting").map(|v| v.len()).unwrap_or(0);
            let created = status_grouped_containers
                .get("created").map(|v| v.len()).unwrap_or(0);
            let configured = status_grouped_containers
                .get("configured").map(|v| v.len()).unwrap_or(0);
            let starting = restarting + created + configured;
            if starting > 0 {
                message.push_str(&format!(
                    "{color}Restarting({number}){reset} ", 
                    color = status_to_color("restarting"),
                    number = starting,
                    reset = style::Reset,
                ));
            }

            let exited = status_grouped_containers
                .get("exited").map(|v| v.len()).unwrap_or(0);
            let dead = status_grouped_containers
                .get("dead").map(|v| v.len()).unwrap_or(0);
            let paused = status_grouped_containers
                .get("paused").map(|v| v.len()).unwrap_or(0);
            let removing = status_grouped_containers
                .get("removing").map(|v| v.len()).unwrap_or(0);
            let stopped = exited + dead + paused + removing;
            if stopped > 0 {
                message.push_str(&format!(
                    "{color}Stopped({number}){reset} ", 
                    color = status_to_color("exited"),
                    number = stopped,
                    reset = style::Reset,
                ));
            }

            println!("{}", message);
        }
        Ok(())
    }
}
