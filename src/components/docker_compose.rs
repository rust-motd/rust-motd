use crate::constants::INDENT_WIDTH;
use async_trait::async_trait;
use docker_api::opts::{ContainerFilter, ContainerListOpts};
use shellexpand;
use std::fs;
use termion::{color, style};

use crate::component::Component;
use crate::components::docker::{new_docker, print_containers, Container};
use crate::config::global_config::GlobalConfig;
use crate::default_prepare;

#[derive(knus::Decode, Debug)]
pub struct ComposeStack {
    #[knus(property)]
    pub path: String,
    #[knus(property)]
    pub display_name: String,
}

#[derive(knus::Decode, Debug)]
pub struct DockerCompose {
    #[knus(children(name = "stack"))]
    pub stacks: Vec<ComposeStack>,
}

#[async_trait]
impl Component for DockerCompose {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        println!("Docker Compose:");
        self.print_or_error()
            .await
            .unwrap_or_else(|err| println!("Docker Compose status error: {err}"));
        println!();
    }
    default_prepare!();
}

impl DockerCompose {
    pub async fn print_or_error(&self) -> Result<(), Box<dyn std::error::Error>> {
        let docker = new_docker()?;

        for ComposeStack { path, display_name } in self.stacks.iter() {
            let path = fs::canonicalize(&*shellexpand::tilde(path))?
                .to_string_lossy()
                .to_string();

            let containers = docker
                .containers()
                .list(
                    &ContainerListOpts::builder()
                        .all(true)
                        .filter([ContainerFilter::Label(
                            "com.docker.compose.project.working_dir".to_string(),
                            path,
                        )])
                        .build(),
                )
                .await?;

            if containers.is_empty() {
                println!(
                    "{indent}{display_name}: {color}Not found{reset}",
                    indent = " ".repeat(INDENT_WIDTH * 2),
                    color = color::Fg(color::Yellow),
                    reset = style::Reset,
                );
                continue;
            }

            let containers = containers
                .into_iter()
                .map(|container| {
                    let name = container
                        .labels
                        .clone()
                        .and_then(|labels| {
                            labels
                                .get("com.docker.compose.service")
                                .map(|n| n.to_string())
                        })
                        .unwrap_or_else(|| {
                            container
                                .names
                                .clone()
                                .map(|names| names[0].to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        });
                    Container {
                        name,
                        summary: container,
                    }
                })
                .collect();

            println!("{indent}{display_name}:", indent = " ".repeat(INDENT_WIDTH));
            print_containers(containers, 2 * INDENT_WIDTH);
        }

        Ok(())
    }
}
