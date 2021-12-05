use crate::constants::INDENT_WIDTH;
use docker_api::{Docker, Result as DockerResult};
use serde::Deserialize;
use serde_plain;
use termion::{color, style};

#[derive(Debug, Deserialize)]
pub struct DockerConfig {}

#[cfg(unix)]
pub fn new_docker() -> DockerResult<Docker> {
    Ok(Docker::unix("/var/run/docker.sock"))
}

#[cfg(not(unix))]
pub fn new_docker() -> DockerResult<Docker> {
    Docker::new("tcp://127.0.0.1:8080")
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum ContainerStatus {
    Created,
    Restarting,
    Running,
    Removing,
    Paused,
    Exited,
    Dead,
}

pub async fn disp_docker(_config: DockerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let docker = new_docker()?;
    let containers = docker.containers().list(&Default::default()).await?;

    // Max length of all the container names (first column)
    // to determine the padding
    let max_len = containers.iter().map(|x| x.names[0].len()).max().unwrap();

    for container in containers {
        let status: ContainerStatus = serde_plain::from_str(&container.state)?;
        let status_color = match status {
            ContainerStatus::Created
            | ContainerStatus::Restarting
            | ContainerStatus::Paused
            | ContainerStatus::Removing => color::Fg(color::Yellow).to_string(),
            ContainerStatus::Running => color::Fg(color::Green).to_string(),
            ContainerStatus::Exited => color::Fg(color::LightBlack).to_string(),
            ContainerStatus::Dead => color::Fg(color::Red).to_string(),
        };
        println!(
            "{}{}: {}{}{}{}",
            " ".repeat(INDENT_WIDTH as usize),
            container.names[0],
            " ".repeat(max_len - container.names[0].len()),
            status_color,
            container.status,
            style::Reset,
        );
    }

    Ok(())
}
