use humantime::format_duration;
use last_rs::{get_logins, Enter, Exit, LastError};
use std::collections::HashMap;
use std::time::Duration;
use termion::{color, style};
use thiserror::Error;
use time::error::Format as TimeFormatError;
use time::error::IndeterminateOffset as TimeIndeterminateOffsetError;
use time::error::InvalidFormatDescription as TimeInvalidFormatDescriptionError;
use time::format_description;
use time::UtcOffset;

use crate::command::BetterCommandError;
use crate::constants::INDENT_WIDTH;
use crate::config::global_config::GlobalConfig;

pub type LastLoginConfig = HashMap<String, usize>;

#[derive(Error, Debug)]
pub enum LastLoginError {
    #[error(transparent)]
    BetterCommand(#[from] BetterCommandError),

    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Last(#[from] LastError),

    #[error(transparent)]
    TimeFormat(#[from] TimeFormatError),

    #[error(transparent)]
    TimeInvalidFormatDescription(#[from] TimeInvalidFormatDescriptionError),

    #[error(transparent)]
    TimeIndeterminateOffset(#[from] TimeIndeterminateOffsetError),
}

fn format_entry(
    entry: &Enter,
    longest_location: usize,
    time_format: &str,
) -> Result<String, LastLoginError> {
    let location = format!("{:>width$}", entry.host, width = longest_location);
    let login_time = entry.login_time;

    let exit = match entry.exit {
        Exit::Logout(time) => {
            // Timezone does not matter here
            // Were taking the difference of two times with the same offset
            let delta_time = time - login_time;
            let delta_time = Duration::new((delta_time.whole_seconds() as u64 / 60) * 60, 0);
            format_duration(delta_time).to_string()
        }
        _ => {
            let (colour, message) = match entry.exit {
                Exit::StillLoggedIn => (color::Fg(color::Green).to_string(), "still logged in"),
                Exit::Crash(_) => (color::Fg(color::Yellow).to_string(), "crash"),
                Exit::Reboot(_) => (color::Fg(color::Yellow).to_string(), "down"),
                Exit::Logout(_) => unreachable!(),
            };
            format!("{}{}{}", colour, message, style::Reset)
        }
    };

    Ok(format!(
        "{indent}from {location} at {login_time} ({exit})",
        location = location,
        login_time = login_time
            .to_offset(UtcOffset::current_local_offset()?)
            .format(&format_description::parse(time_format)?)?,
        exit = exit,
        indent = " ".repeat(2 * INDENT_WIDTH as usize),
    ))
}

pub fn disp_last_login(
    config: LastLoginConfig,
    global_config: &GlobalConfig,
) -> Result<(), LastLoginError> {
    println!("Last Login:");

    for (username, num_logins) in config {
        println!("{}{}:", " ".repeat(INDENT_WIDTH as usize), username);
        let entries = get_logins("/var/log/wtmp")?
            .into_iter()
            .filter(|entry| entry.user == username)
            .take(num_logins)
            .collect::<Vec<Enter>>();

        let longest_location = entries.iter().map(|entry| entry.host.len()).max();
        match longest_location {
            Some(longest_location) => {
                let formatted_entries = entries.iter().map(|entry| {
                    format_entry(entry, longest_location, &global_config.time_format)
                });
                for entry in formatted_entries {
                    match entry {
                        Ok(x) => println!("{}", x),
                        Err(err) => println!("{}", err),
                    }
                }
            }
            None => println!(
                "{indent}{color}No logins found for `{username}'{reset}",
                indent = " ".repeat(2 * INDENT_WIDTH as usize),
                username = username,
                color = color::Fg(color::Red),
                reset = style::Reset,
            ),
        }
    }

    Ok(())
}
