use async_trait::async_trait;
use indexmap::IndexMap;
use termion::{color, style};
use thiserror::Error;

use crate::command::{BetterCommand, BetterCommandError};
use crate::component::Component;
use crate::config::global_config::GlobalConfig;
use crate::constants::INDENT_WIDTH;
use crate::default_prepare;

pub struct ServiceStatus {
    pub services: IndexMap<String, String>,
}

pub struct UserServiceStatus {
    pub services: IndexMap<String, String>,
}

#[async_trait]
impl Component for ServiceStatus {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        println!("System Services:");
        print_or_error(&self.services, false)
            .unwrap_or_else(|err| println!("Service status error: {err}"));
        println!();
    }
    default_prepare!();
}

#[async_trait]
impl Component for UserServiceStatus {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        println!("User Services:");
        print_or_error(&self.services, true)
            .unwrap_or_else(|err| println!("User service status error: {err}"));
        println!();
    }
    default_prepare!();
}

#[derive(Error, Debug)]
pub enum ServiceStatusError {
    #[error("Empty configuration for system services. Please remove the entire block to disable this component.")]
    ConfigEmpty,

    #[error(transparent)]
    BetterCommand(#[from] BetterCommandError),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

fn get_service_status(service: &str, user: bool) -> Result<String, ServiceStatusError> {
    let executable = "systemctl";
    let mut args = vec!["is-active", service];

    if user {
        args.insert(0, "--user");
    }

    let output = BetterCommand::new(executable)
        .args(args)
        .get_output_string()?;

    Ok(output.split_whitespace().collect())
}

pub fn print_or_error(
    config: &IndexMap<String, String>,
    user: bool,
) -> Result<(), ServiceStatusError> {
    if config.is_empty() {
        return Err(ServiceStatusError::ConfigEmpty);
    }

    let padding = config.keys().map(|x| x.len()).max().unwrap();

    for key in config.keys() {
        let status = get_service_status(config.get(key).unwrap(), user)?;

        let status_color = match status.as_ref() {
            "active" => color::Fg(color::Green).to_string(),
            "inactive" => color::Fg(color::Yellow).to_string(),
            "failed" => color::Fg(color::Red).to_string(),
            _ => style::Reset.to_string(),
        };

        println!(
            "{}{}: {}{}{}{}",
            " ".repeat(INDENT_WIDTH),
            key,
            " ".repeat(padding - key.len()),
            status_color,
            status,
            style::Reset,
        );
    }

    Ok(())
}
