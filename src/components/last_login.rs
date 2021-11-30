use humantime::format_duration;
use last_rs::{get_logins, Enter, Exit, LastError};
use std::collections::HashMap;
use std::time::Duration;
use termion::{color, style};
use thiserror::Error;
use time;

use crate::command::BetterCommandError;
use crate::constants::{GlobalSettings, INDENT_WIDTH};

pub type LastLoginCfg = HashMap<String, usize>;

#[derive(Error, Debug)]
pub enum LastLoginError {
    #[error(transparent)]
    BetterCommand(#[from] BetterCommandError),

    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),

    #[error(transparent)]
    ChronoOutOfRange(#[from] time::OutOfRangeError),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Last(#[from] LastError),
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
            let delta_time = (time - login_time).to_std()?;
            let delta_time = Duration::new((delta_time.as_secs() / 60) * 60, 0);
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
        login_time = login_time.format(time_format),
        exit = exit,
        indent = " ".repeat(2 * INDENT_WIDTH as usize),
    ))
}

pub fn disp_last_login(
    config: LastLoginCfg,
    global_settings: &GlobalSettings,
) -> Result<(), LastLoginError> {
    println!("Last Login:");

    for (username, num_logins) in config {
        println!("{}{}:", " ".repeat(INDENT_WIDTH as usize), username);
        let entries = get_logins("/var/log/wtmp")?
            .into_iter()
            .filter(|entry| entry.user == username)
            .take(num_logins)
            .collect::<Vec<Enter>>();

        let longest_location = entries.iter().map(|entry| entry.host.len()).max().unwrap();
        let formatted_entries = entries
            .iter()
            .map(|entry| format_entry(entry, longest_location, &global_settings.time_format));
        for entry in formatted_entries {
            match entry {
                Ok(x) => println!("{}", x),
                Err(err) => println!("{}", err),
            }
        }
    }

    Ok(())
}
