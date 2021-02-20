use crate::constants::INDENT_WIDTH;
use chrono::DateTime;
use regex::Regex;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::process::Command;
use termion::{color, style};

pub type LastLoginCfg = HashMap<String, usize>;

#[derive(Debug)]
struct Entry<'a> {
    username: &'a str,
    location: &'a str,
    start_time: &'a str,
    end_time: &'a str,
}

fn parse_entry(line: &str) -> Entry {
    // TODO: Use lazy_static
    let separator_regex = Regex::new(r"(?:\s{2,})|(?:\s-\s)").unwrap();

    let items = separator_regex.split(line).collect::<Vec<_>>();
    Entry {
        username: items[0],
        location: items[2],
        start_time: items[3],
        end_time: items[4],
    }
}

fn format_entry(entry: &Entry, longest_location: usize) -> Result<String, chrono::ParseError> {
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

pub fn disp_last_login(config: LastLoginCfg) -> Result<(), std::io::Error> {
    // TODO: Clean this up

    println!();
    println!("Last Login:");

    for (username, num_logins) in config {
        println!("{}{}:", " ".repeat(INDENT_WIDTH as usize), username);

        // Use `last` command to get last logins
        let output = Command::new("last")
            .arg("--time-format=iso")
            .arg(&username)
            .output()?
            .stdout;

        // Output to string
        let output = String::from_utf8_lossy(&output);

        // Split lines and take desigred number
        let mut output = output
            .lines()
            .filter(|line| line.starts_with(&username))
            .take(num_logins)
            .peekable();

        if output.peek().is_none() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Could not find any logins for user {}", username),
            ));
        }

        let entries = output.map(parse_entry).collect::<Vec<Entry>>();
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
