use chrono::{Duration, TimeZone, Utc};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use termion::{color, style};
use thiserror::Error;

use crate::command::{BetterCommand, BetterCommandError};
use crate::constants::INDENT_WIDTH;

#[derive(Debug, Deserialize)]
pub struct SSLCertsCfg {
    sort_method: Option<SortMethod>,
    certs: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
enum SortMethod {
    #[serde(alias = "alphabetical")] // Alias used to match lowercase spelling as well
    Alphabetical,
    #[serde(alias = "expiration")] // Alias used to match lowercase spelling as well
    Expiration,
}

#[derive(Error, Debug)]
pub enum SSLCertsError {
    #[error("Failed to parse timestamp")]
    ChronoParseError(#[from] chrono::ParseError),

    #[error(transparent)]
    BetterCommandError(#[from] BetterCommandError),

    #[error("Failed to compile Regex")]
    RegexError(#[from] regex::Error),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

struct CertInfo {
    name: String,
    status: String,
    expiration: systemstat::DateTime<systemstat::Utc>,
}

pub fn disp_ssl(config: SSLCertsCfg) -> Result<(), SSLCertsError> {
    // TODO: Support time zone
    // chrono does not support %Z
    let re = Regex::new(r"notAfter=([A-Za-z]+ +\d+ +[\d:]+ +\d{4}) +[A-Za-z]+\n")?;
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

        match re.captures(&output) {
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

    if let Some(sort_method) = config.sort_method {
        match sort_method {
            SortMethod::Alphabetical => {
                cert_infos.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SortMethod::Expiration => {
                cert_infos.sort_by(|a, b| a.expiration.cmp(&b.expiration));
            }
        }
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
