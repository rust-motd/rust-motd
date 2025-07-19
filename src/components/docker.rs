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

pub struct Container {
    pub summary: ContainerSummary,
    pub name: String,
}

pub fn init_api(socket: &str) -> DockerResult<DockerAPI> {
    DockerAPI::new(socket)
}

pub fn print_containers(containers: Vec<Container>, indent_width: usize, padding: usize) {
    for container in containers {
        let status_color = match container.summary.state.map(|s| s.to_lowercase()).as_deref() {
            Some("created") | Some("restarting") | Some("paused") | Some("removing")
            | Some("configured") => color::Fg(color::Yellow).to_string(),
            Some("running") => color::Fg(color::Green).to_string(),
            Some("exited") => color::Fg(color::LightBlack).to_string(),
            Some("dead") => color::Fg(color::Red).to_string(),
            _ => color::Fg(color::White).to_string(),
        };
        println!(
            "{indent}{name}: {padding}{color}{status}{reset}",
            indent = " ".repeat(indent_width),
            name = container.name,
            padding = " ".repeat(padding - container.name.len()),
            color = status_color,
            status = container.summary.status.unwrap_or(String::from("?")),
            reset = style::Reset,
        );
    }
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
        let containers: Vec<Container> = self
            .containers
            .iter()
            .filter_map(
                |DockerContainer { docker_name, display_name }| match summary_hash.get(docker_name) {
                    Some(&summary) => Some(Container {
                        name: display_name.clone(),
                        summary: summary.clone(),
                    }),
                    None => {
                        println!(
                            "{indent}{color}Warning: Could not find container `{docker_name}'{reset}",
                            indent = " ".repeat(INDENT_WIDTH),
                            color = color::Fg(color::Yellow),
                            docker_name = docker_name,
                            reset = style::Reset
                        );
                        None
                    }
                },
            )
            .collect();

        // Max length of all the container names (first column)
        // to determine the padding
        let max_container_name = containers
            .iter()
            .map(|container| container.name.len())
            .max()
            .unwrap_or(0);

        print_containers(containers, INDENT_WIDTH, max_container_name);

        Ok(())
    }
}
