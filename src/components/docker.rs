use crate::constants::INDENT_WIDTH;
use async_trait::async_trait;
use docker_api::models::ContainerSummary;
use docker_api::opts::ContainerListOpts;
use docker_api::{Docker as DockerAPI, Result as DockerResult};
use std::collections::HashMap;
use termion::{color, style};
use unicode_width::UnicodeWidthStr;

use crate::component::Component;
use crate::config::global_config::GlobalConfig;
use crate::default_prepare;

#[cfg(unix)]
pub const DEFAULT_SOCKET: &str = "unix:///var/run/docker.sock";

#[cfg(not(unix))]
pub const DEFAULT_SOCKET: &str = "tcp://127.0.0.1:8080";

const DEFAULT_TITLE: &str = "Docker";

#[derive(knus::Decode, Debug)]
pub struct DockerContainer {
    #[knus(property)]
    pub docker_name: String,
    #[knus(property)]
    pub display_name: String,
}

#[derive(knus::Decode, Debug)]
pub struct Docker {
    #[knus(children(name = "container"))]
    pub containers: Vec<DockerContainer>,

    #[knus(property, default=DEFAULT_TITLE.into())]
    pub title: String,

    #[knus(property, default=DEFAULT_SOCKET.into())]
    pub socket: String,
}

#[async_trait]
impl Component for Docker {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        println!("{}:", self.title);
        self.print_or_error()
            .await
            .unwrap_or_else(|err| println!("Docker status error: {err}"));
        println!();
    }
    default_prepare!();
}

#[derive(Debug)]
pub struct Container {
    pub summary: ContainerSummary,
    pub name: String,
}

pub fn init_api(socket: &str) -> DockerResult<DockerAPI> {
    DockerAPI::new(socket)
}

pub fn state_to_color(state: &str) -> String {
    match state {
        "created" | "restarting" | "paused" | "removing" | "configured" => {
            color::Fg(color::Yellow).to_string()
        }
        "running" => color::Fg(color::Green).to_string(),
        "exited" => color::Fg(color::LightBlack).to_string(),
        "dead" => color::Fg(color::Red).to_string(),
        _ => color::Fg(color::White).to_string(),
    }
}

pub fn print_container(container: Container, indent_width: usize, padding: usize) {
    let status_color = state_to_color(
        container
            .summary
            .state
            .map(|s| s.to_lowercase())
            .as_deref()
            .unwrap_or(""),
    );
    println!(
        "{indent}{name}: {padding}{color}{status}{reset}",
        indent = " ".repeat(indent_width),
        name = container.name,
        padding = " ".repeat(padding - container.name.width()),
        color = status_color,
        status = container.summary.status.unwrap_or(String::from("?")),
        reset = style::Reset,
    );
}

impl Docker {
    pub fn new(containers: Vec<DockerContainer>) -> Self {
        Docker {
            title: DEFAULT_TITLE.into(),
            socket: DEFAULT_SOCKET.to_string(),
            containers,
        }
    }

    pub async fn print_or_error(&self) -> Result<(), Box<dyn std::error::Error>> {
        let api = init_api(&self.socket)?;

        // Get all containers from library
        let container_summaries = api
            .containers()
            .list(&ContainerListOpts::builder().all(true).build())
            .await?;
        // Since a container can have more than one name, make a hash indexed
        // by name to look up based on the name in the config file
        let summary_hash: HashMap<String, &ContainerSummary> = container_summaries
            .iter()
            .flat_map(|container_summary| {
                container_summary.names.iter().flat_map(move |names| {
                    names
                        .iter()
                        .map(move |name| (name.clone(), container_summary))
                })
            })
            .collect();
        let containers: Vec<Result<Container, &String>> = self
            .containers
            .iter()
            .map(|container| match summary_hash.get(&container.docker_name) {
                Some(&summary) => Ok(Container {
                    name: container.display_name.clone(),
                    summary: summary.clone(),
                }),
                None => Err(&container.docker_name),
            })
            .collect();

        // Max length of all the container names (first column)
        // to determine the padding
        let max_container_name = containers
            .iter()
            .flatten()
            .map(|container| container.name.width())
            .max()
            .unwrap_or(0);

        for container in containers {
            match container {
                Ok(container) => print_container(container, INDENT_WIDTH, max_container_name),
                Err(docker_name) => println!(
                    "{indent}{color}Warning: Could not find container `{docker_name}'{reset}",
                    indent = " ".repeat(INDENT_WIDTH),
                    color = color::Fg(color::Yellow),
                    docker_name = docker_name,
                    reset = style::Reset
                ),
            }
        }

        Ok(())
    }
}
