use itertools::Itertools;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::process::Command;
use termion::{color, style};
use thiserror::Error;

use crate::constants::INDENT_WIDTH;

pub type ServiceStatusCfg = HashMap<String, String>;

#[derive(Error, Debug)]
pub enum ServiceStatusError {
    #[error("Empty configuration for system services.")]
    ConfigEmtpyError,

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

fn get_service_status(service: &str, user: bool) -> Result<String, ServiceStatusError> {
    let executable = "systemctl";
    let mut args = vec!["is-active", service];

    if user {
        args.insert(0, "--user");
    }

    let output = Command::new(executable)
        .args(args)
        .output()
        // TODO: Try to clean this up
        .map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                return std::io::Error::new(
                    ErrorKind::NotFound,
                    format!("Command not found: {}", executable),
                );
            }
            err
        })?;

    Ok(String::from_utf8_lossy(&output.stdout)
        .into_owned()
        .split_whitespace()
        .collect())
}

pub fn disp_service_status(config: ServiceStatusCfg, user: bool) -> Result<(), ServiceStatusError> {
    if config.is_empty() {
        return Err(ServiceStatusError::ConfigEmtpyError);
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
