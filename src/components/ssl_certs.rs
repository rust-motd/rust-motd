use chrono::{Duration, TimeZone, Utc};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::process::Command;
use termion::{color, style};
use thiserror::Error;

use crate::constants::INDENT_WIDTH;

#[derive(Debug, Deserialize)]
pub struct SSLCertsCfg {
    sort_method: String, // TODO: Maybe switch to enum insead of string
    // need to figure out how to do this in Serde
    // Also TODO: Implement this ^
    certs: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum SSLCertsError {
    #[error("Failed to parse timestamp")]
    ChronoParseError(#[from] chrono::ParseError),

    // TODO: The executable should be configurable too
    #[error("openssl failed with exit code {exit_code:?}:\n{error:?}")]
    CommandError { exit_code: i32, error: String },

    #[error("Failed to compile Regex")]
    RegexError(#[from] regex::Error),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub fn disp_ssl(config: SSLCertsCfg) -> Result<(), SSLCertsError> {
    // TODO: Support time zone
    // chrono does not support %Z
    let re = Regex::new(r"notAfter=([A-Za-z]+ +\d+ +[\d:]+ +\d{4}) +[A-Za-z]+\n")?;

    println!("SSL Certificates:");
    for (name, path) in config.certs {
        let executable = "openssl";
        let output = Command::new(executable)
            .arg("x509")
            .arg("-in")
            .arg(&path)
            .arg("-dates")
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
            return Err(SSLCertsError::CommandError {
                exit_code: output.status.code().unwrap(),
                error: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let output = String::from_utf8_lossy(&output.stdout);
        match re.captures(&output) {
            Some(captures) => {
                let date = Utc.datetime_from_str(&captures[1], "%B %_d %T %Y")?;

                let now = Utc::now();
                let status = if date < now {
                    format!("{}expired on{}", color::Fg(color::Red), style::Reset)
                } else if date < now + Duration::days(30) {
                    format!("{}expiring on{}", color::Fg(color::Yellow), style::Reset)
                } else {
                    format!("{}valid until{}", color::Fg(color::Green), style::Reset)
                };
                println!(
                    "{}{} {} {}",
                    " ".repeat(INDENT_WIDTH as usize),
                    name,
                    status,
                    date
                );
            }
            None => println!(
                "{}Error parsing certificate {}",
                " ".repeat(INDENT_WIDTH as usize),
                name
            ),
        }
    }

    Ok(())
}
