use crate::constants::INDENT_WIDTH;
use chrono::DateTime;
use regex::Regex;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::process::Command;
use termion::{color, style};
use thiserror::Error;

pub type LastLoginCfg = HashMap<String, usize>;

#[derive(Debug)]
struct Entry<'a> {
    username: &'a str,
    location: &'a str,
    start_time: &'a str,
    end_time: &'a str,
}

#[derive(Error, Debug)]
pub enum LastLoginError {
    // TODO: The executable should be configurable too
    #[error("last failed with exit code {exit_code:?}:\n{error:?}")]
    CommandError { exit_code: i32, error: String },

    #[error("Could not find any logins for user {username:?}")]
    NoUser { username: String },

    #[error("Failed to parse output from `last`")]
    ParseError,

    #[error(transparent)]
    ChronoParseError(#[from] chrono::ParseError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

fn parse_entry(line: &str) -> Result<Entry, LastLoginError> {
    // TODO: Use lazy_static
    let separator_regex = Regex::new(r"(?:\s{2,})|(?:\s-\s)").unwrap();

    let items = separator_regex.split(line).collect::<Vec<_>>();

    if items.len() < 5 {
        return Err(LastLoginError::ParseError);
    }

    Ok(Entry {
        username: items[0],
        location: items[2],
        start_time: items[3],
        end_time: items[4],
    })
}

fn format_entry(entry: &Entry, longest_location: usize) -> Result<String, LastLoginError> {
    let location = format!("{:>width$}", entry.location, width = longest_location);
    let start_time = DateTime::parse_from_rfc3339(entry.start_time)?;

    let end_time = match DateTime::parse_from_rfc3339(entry.end_time) {
        Ok(end_time) => format!("{} minutes", (end_time - start_time).num_minutes()),
        Err(_) => {
            let colour = if entry.end_time == "still logged in" {
                format!("{}", color::Fg(color::Green))
            } else {
                format!("{}", color::Fg(color::Yellow))
            };
            format!("{}{}{}", colour, entry.end_time.to_string(), style::Reset)
        }
    };

    Ok(format!(
        "{indent}from {location} at {start_time} ({end_time})",
        location = location,
        start_time = start_time.format("%m/%d/%Y %I:%M%p"),
        end_time = end_time,
        indent = " ".repeat(2 * INDENT_WIDTH as usize),
    ))
}

pub fn disp_last_login(config: LastLoginCfg) -> Result<(), LastLoginError> {
    // TODO: Clean this up

    println!();
    println!("Last Login:");

    for (username, num_logins) in config {
        println!("{}{}:", " ".repeat(INDENT_WIDTH as usize), username);

        // Use `last` command to get last logins
        let executable = "last";
        let output = Command::new(executable)
            // Sometimes last doesn't show location otherwise for some reason
            .arg("--ip")
            .arg("--time-format=iso")
            .arg(&username)
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
            return Err(LastLoginError::CommandError {
                exit_code: output.status.code().unwrap(),
                error: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        // Output to string
        let output = String::from_utf8_lossy(&output.stdout);

        // Split lines and take desigred number
        let mut output = output
            .lines()
            .filter(|line| line.starts_with(&username))
            .take(num_logins)
            .peekable();

        if output.peek().is_none() {
            return Err(LastLoginError::NoUser { username });
        }

        let entries = output
            .map(parse_entry)
            .collect::<Result<Vec<Entry>, LastLoginError>>()?;
        let longest_location = entries
            .iter()
            .map(|entry| entry.location.len())
            .max()
            .unwrap();
        let formatted_entries = entries
            .iter()
            .map(|entry| format_entry(entry, longest_location));
        for entry in formatted_entries {
            match entry {
                Ok(x) => println!("{}", x),
                Err(err) => println!("{}", err),
            }
        }
    }

    Ok(())
}
