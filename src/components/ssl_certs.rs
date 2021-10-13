use chrono::{Duration, TimeZone, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use termion::{color, style};
use thiserror::Error;

use crate::command::{BetterCommand, BetterCommandError};
use crate::constants::INDENT_WIDTH;

#[derive(Debug, Deserialize)]
pub struct SSLCertsCfg {
    #[serde(default)]
    sort_method: SortMethod,
    certs: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
enum SortMethod {
    #[serde(alias = "alphabetical")] // Alias used to match lowercase spelling as well
    Alphabetical,
    #[serde(alias = "expiration")] // Alias used to match lowercase spelling as well
    Expiration,
    #[serde(alias = "manual")] // Alias used to match lowercase spelling as well
    Manual,
}

impl Default for SortMethod {
    fn default() -> Self {
        SortMethod::Manual
    }
}

#[derive(Error, Debug)]
pub enum SSLCertsError {
    #[error("Failed to parse timestamp")]
    ChronoParse(#[from] chrono::ParseError),

    #[error(transparent)]
    BetterCommand(#[from] BetterCommandError),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

struct CertInfo {
    name: String,
    status: String,
    expiration: systemstat::DateTime<systemstat::Utc>,
}

pub fn disp_ssl(config: SSLCertsCfg) -> Result<(), SSLCertsError> {
    // TODO: Support time zone
    // chrono does not support %Z

    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"notAfter=([A-Za-z]+ +\d+ +[\d:]+ +\d{4}) +[A-Za-z]+\n").unwrap();
    }
    let mut cert_infos: Vec<CertInfo> = Vec::new();

    println!("SSL Certificates:");
    for (name, path) in config.certs {
        let executable = "openssl";
        let output = BetterCommand::new(executable)
            .arg("x509")
            .arg("-in")
            .arg(&path)
            .arg("-dates")
            .check_status_and_get_output_string()?;

        match RE.captures(&output) {
            Some(captures) => {
                let expiration = Utc.datetime_from_str(&captures[1], "%B %_d %T %Y")?;

                let now = Utc::now();
                let status = if expiration < now {
                    format!("{}expired on{}", color::Fg(color::Red), style::Reset)
                } else if expiration < now + Duration::days(30) {
                    format!("{}expiring on{}", color::Fg(color::Yellow), style::Reset)
                } else {
                    format!("{}valid until{}", color::Fg(color::Green), style::Reset)
                };
                cert_infos.push(CertInfo {
                    name,
                    status,
                    expiration,
                });
            }
            None => println!(
                "{}Error parsing certificate {}",
                " ".repeat(INDENT_WIDTH as usize),
                name
            ),
        }
    }

    match config.sort_method {
        SortMethod::Alphabetical => {
            cert_infos.sort_by(|a, b| a.name.cmp(&b.name));
        }
        SortMethod::Expiration => {
            cert_infos.sort_by(|a, b| a.expiration.cmp(&b.expiration));
        }
        SortMethod::Manual => {}
    }

    for cert_info in cert_infos.into_iter() {
        println!(
            "{}{} {} {}",
            " ".repeat(INDENT_WIDTH as usize),
            cert_info.name,
            cert_info.status,
            cert_info.expiration
        );
    }

    Ok(())
}
