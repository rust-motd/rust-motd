use crate::constants::INDENT_WIDTH;
use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

use crate::command::{BetterCommand, BetterCommandError};
use crate::component::Component;
use crate::config::global_config::GlobalConfig;
use crate::default_prepare;

#[derive(knus::Decode, Debug, Deserialize)]
pub struct Fail2Ban {
    #[knus(children(name = "jail"), unwrap(argument))]
    jails: Vec<String>,
}

#[async_trait]
impl Component for Fail2Ban {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        self.print_or_error()
            .unwrap_or_else(|err| println!("Fail2Ban error: {err}"));
        println!();
    }
    default_prepare!();
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

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

fn get_jail_status(jail: &str) -> Result<Entry, Fail2BanError> {
    lazy_static! {
        static ref TOTAL_REGEX: Regex = Regex::new(r"Total banned:\s+([0-9]+)").unwrap();
        static ref CURRENT_REGEX: Regex = Regex::new(r"Currently banned:\s+([0-9]+)").unwrap();
    }

    let executable = "fail2ban-client";
    let output = BetterCommand::new(executable)
        .arg("status")
        .arg(jail)
        .check_status_and_get_output_string()?;

    let total = TOTAL_REGEX.captures_iter(&output).next().unwrap()[1].parse::<u32>()?;
    let current = CURRENT_REGEX.captures_iter(&output).next().unwrap()[1].parse::<u32>()?;

    Ok(Entry { total, current })
}

impl Fail2Ban {
    pub fn print_or_error(self) -> Result<(), Fail2BanError> {
        println!("Fail2Ban:");

        for jail in self.jails {
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
                indent = " ".repeat(INDENT_WIDTH),
            );
        }

        Ok(())
    }
}
