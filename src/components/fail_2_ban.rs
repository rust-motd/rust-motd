use crate::constants::INDENT_WIDTH;
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

use crate::command::{BetterCommand, BetterCommandError};

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
    #[error(transparent)]
    BetterCommand(#[from] BetterCommandError),

    #[error("Failed to parse int in output")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Failed to compile Regex")]
    Regex(#[from] regex::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

fn get_jail_status(jail: &str) -> Result<Entry, Fail2BanError> {
    let executable = "fail2ban-client";
    let output = BetterCommand::new(executable)
        .arg("status")
        .arg(jail)
        .check_status_and_get_output_string()?;

    // TODO: Use lazy_static
    let total_regex = Regex::new(r"Total banned:\s+([0-9]+)")?;
    let current_regex = Regex::new(r"Currently banned:\s+([0-9]+)")?;
    let total = total_regex.captures_iter(&output).next().unwrap()[1].parse::<u32>()?;
    let current = current_regex.captures_iter(&output).next().unwrap()[1].parse::<u32>()?;

    Ok(Entry { total, current })
}

pub fn disp_fail_2_ban(config: Fail2BanCfg) -> Result<(), Fail2BanError> {
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
