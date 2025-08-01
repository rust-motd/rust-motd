use async_trait::async_trait;
use docker_api::opts::{ContainerFilter, ContainerListOpts};
use itertools::Itertools;
use shellexpand;
use std::fs;
use termion::{color, style};

use crate::component::Component;
use crate::components::docker::{
    init_api, print_containers, state_to_color, Container, DEFAULT_SOCKET,
};
use crate::config::global_config::GlobalConfig;
use crate::constants::INDENT_WIDTH;
use crate::default_prepare;

const DEFAULT_TITLE: &str = "Docker Compose";

#[derive(knus::DecodeScalar, Debug, Default)]
pub enum DockerComposeStyle {
    #[default]
    Count,
    Full,
}

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

    #[knus(property, default=DEFAULT_TITLE.into())]
    pub title: String,

    #[knus(property, default=DEFAULT_SOCKET.into())]
    pub socket: String,

    #[knus(property, default=DockerComposeStyle::default())]
    pub style: DockerComposeStyle,
}

#[derive(Debug)]
struct PreparedStack {
    display_name: String,

    max_container_name: usize,

    containers: Vec<Container>,
}

// Information for grouping similar states together (like created / restarting / paused ...)
struct SimilarStates {
    title: &'static str,
    states: &'static [&'static str],
}

static SIMILAR_STATES: &[SimilarStates] = &[
    SimilarStates {
        title: "Running",
        states: &["running"],
    },
    SimilarStates {
        title: "Restarting",
        states: &["restarting", "configured", "created"],
    },
    SimilarStates {
        title: "Stopped",
        states: &["exited", "dead", "paused", "removing"],
    },
];

#[async_trait]
impl Component for DockerCompose {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        println!("{}:", self.title);
        self.print_or_error()
            .await
            .unwrap_or_else(|err| println!("{} status error: {}", self.title, err));
        println!();
    }
    default_prepare!();
}

impl DockerCompose {
    pub fn new(stacks: Vec<ComposeStack>) -> Self {
        DockerCompose {
            title: DEFAULT_TITLE.into(),
            socket: DEFAULT_SOCKET.to_string(),
            style: DockerComposeStyle::default(),
            stacks,
        }
    }

    pub async fn print_or_error(&self) -> Result<(), Box<dyn std::error::Error>> {
        let api = init_api(&self.socket)?;

        let mut prepared_stacks: Vec<PreparedStack> = vec![];

        for ComposeStack { path, display_name } in self.stacks.iter() {
            let path = fs::canonicalize(&*shellexpand::tilde(path))?
                .to_string_lossy()
                .to_string();

            let containers = api
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
                    display_name = display_name.clone(),
                    color = color::Fg(color::Yellow),
                    reset = style::Reset,
                );
                continue;
            }

            let containers: Vec<Container> = containers
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

            let max_container_name = containers
                .iter()
                .map(|container| container.name.len())
                .max()
                .unwrap_or(0);

            prepared_stacks.push(PreparedStack {
                display_name: display_name.clone(),
                max_container_name,
                containers,
            });
        }

        let max_container_name = prepared_stacks
            .iter()
            .map(|stack| stack.max_container_name)
            .max()
            .unwrap_or(0);

        match self.style {
            DockerComposeStyle::Full => self.print_full(prepared_stacks, max_container_name),
            DockerComposeStyle::Count => self.print_count(prepared_stacks),
        };

        Ok(())
    }

    fn print_full(&self, prepared_stacks: Vec<PreparedStack>, max_container_name: usize) {
        for prepared_stack in prepared_stacks.into_iter() {
            println!(
                "{indent}{}:",
                prepared_stack.display_name,
                indent = " ".repeat(INDENT_WIDTH)
            );
            print_containers(
                prepared_stack.containers,
                2 * INDENT_WIDTH,
                max_container_name,
            );
        }
    }

    fn print_count(&self, prepared_stacks: Vec<PreparedStack>) {
        let longest_display_name = prepared_stacks
            .iter()
            .map(|stack| stack.display_name.len())
            .max()
            .unwrap_or(0);

        for prepared_stack in prepared_stacks.into_iter() {
            let grouped = prepared_stack
                .containers
                .iter()
                .map(|container| {
                    (
                        container
                            .summary
                            .state
                            .clone()
                            .unwrap_or("unknown".to_owned()),
                        container,
                    )
                })
                .into_group_map()
                .into_iter()
                .map(|(k, v)| (k, v.len()))
                .collect::<std::collections::HashMap<String, usize>>();

            let states = SIMILAR_STATES
                .iter()
                .flat_map(|similar_states| {
                    let count = similar_states
                        .states
                        .iter()
                        .map(|state| grouped.get(&**state).unwrap_or(&0))
                        .sum::<usize>();
                    if count == 0 {
                        return None;
                    }
                    Some(format!(
                        "{color}{count} {title}{reset}",
                        color = state_to_color(similar_states.states[0]),
                        title = similar_states.title,
                        count = count,
                        reset = style::Reset,
                    ))
                })
                .join(" ");

            println!(
                "{indent}{name}:{padding} {states}",
                indent = " ".repeat(INDENT_WIDTH),
                name = prepared_stack.display_name,
                padding = " ".repeat(longest_display_name - prepared_stack.display_name.len()),
                states = states,
            );
        }
    }
}
