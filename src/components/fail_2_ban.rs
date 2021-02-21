use crate::constants::INDENT_WIDTH;
use regex::Regex;
use serde::Deserialize;
use std::io::ErrorKind;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Fail2BanCfg {
    jails: Vec<String>,
}

struct Entry {
    total: u32,
    current: u32,
}

#[derive(Error, Debug)]
pub enum Fail2BanError {
    // TODO: The executable should be configurable too
    #[error("fail2ban-client failed with exit code {exit_code:?}:\n{error:?}")]
    CommandError { exit_code: i32, error: String },

    #[error("Failed to parse int in output")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Failed to compile Regex")]
    RegexError(#[from] regex::Error),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

fn get_jail_status(jail: &str) -> Result<Entry, Fail2BanError> {
    let executable = "fail2ban-client";
    let output = Command::new(executable)
        .arg("status")
        .arg(jail)
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

    if !output.status.success() {
        return Err(Fail2BanError::CommandError {
            exit_code: output.status.code().unwrap(),
            error: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    let output = String::from_utf8_lossy(&output.stdout);

    // TODO: Use lazy_static
    let total_regex = Regex::new(r"Total banned:\s+([0-9]+)")?;
    let current_regex = Regex::new(r"Currently banned:\s+([0-9]+)")?;

    Ok(Entry {
        total: total_regex.captures_iter(&output).next().unwrap()[1].parse::<u32>()?,
        current: current_regex.captures_iter(&output).next().unwrap()[1].parse::<u32>()?,
    })
}

pub fn disp_fail_2_ban(config: Fail2BanCfg) -> Result<(), Fail2BanError> {
    println!();
    println!("Fail2Ban:");

    for jail in config.jails {
        let entry = get_jail_status(&jail)?;
        println!(
            concat!(
                "{indent}{jail}:\n",
                "{indent}{indent}Total bans:   {total}\n",
                "{indent}{indent}Current bans: {current}",
            ),
            jail = jail,
            total = entry.total,
            current = entry.current,
            indent = " ".repeat(INDENT_WIDTH as usize),
        );
    }

    Ok(())
}
