use itertools::Itertools;
use std::collections::HashMap;
use termion::{color, style};
use thiserror::Error;

use crate::command::{BetterCommand, BetterCommandError};
use crate::constants::INDENT_WIDTH;

pub type ServiceStatusConfig = HashMap<String, String>;

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

pub fn disp_service_status(config: ServiceStatusConfig, user: bool) -> Result<(), ServiceStatusError> {
    if config.is_empty() {
        return Err(ServiceStatusError::ConfigEmpty);
    }

    let padding = config.keys().map(|x| x.len()).max().unwrap();

    for key in config.keys().sorted() {
        let status = get_service_status(config.get(key).unwrap(), user)?;

        let status_color = match status.as_ref() {
            "active" => color::Fg(color::Green).to_string(),
            "inactive" => color::Fg(color::Yellow).to_string(),
            "failed" => color::Fg(color::Red).to_string(),
            _ => style::Reset.to_string(),
        };

        println!(
            "{}{}: {}{}{}{}",
            " ".repeat(INDENT_WIDTH as usize),
            key,
            " ".repeat(padding - key.len()),
            status_color,
            status,
            style::Reset,
        );
    }

    Ok(())
}
