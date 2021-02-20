use itertools::Itertools;
use std::collections::HashMap;
use std::process::Command;
use termion::{color, style};

use crate::constants::INDENT_WIDTH;

pub type ServiceStatusCfg = HashMap<String, String>;

fn get_service_status(service: &str) -> Result<String, std::io::Error> {
    let output = Command::new("systemctl")
        .arg("is-active")
        .arg(service)
        .output()?
        .stdout;
    Ok(String::from_utf8_lossy(&output)
        .into_owned()
        .split_whitespace()
        .collect())
}

pub fn disp_service_status(config: ServiceStatusCfg) -> Result<(), std::io::Error> {
    let padding = config.keys().map(|x| x.len()).max().unwrap();

    println!();
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
