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
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

fn get_service_status(service: &str) -> Result<String, ServiceStatusError> {
    let executable = "systemctl";
    let output = Command::new(executable)
        .arg("is-active")
        .arg(service)
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

pub fn disp_service_status(config: ServiceStatusCfg) -> Result<(), ServiceStatusError> {
    let padding = config.keys().map(|x| x.len()).max().unwrap();

    println!("Services:");
    for key in config.keys().sorted() {
        let status = get_service_status(config.get(key).unwrap())?;

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
