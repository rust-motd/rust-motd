use chrono::{Duration, TimeZone, Utc};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;
use termion::{color, style};

use crate::constants::INDENT_WIDTH;

#[derive(Debug, Deserialize)]
pub struct SSLCertsCfg {
    sort_method: String, // TODO: Maybe switch to enum insead of string
    // need to figure out how to do this in Serde
    // Also TODO: Implement this ^
    certs: HashMap<String, String>,
}

pub fn disp_ssl(config: SSLCertsCfg) -> Result<(), chrono::ParseError> {
    // TODO: Support time zone
    // chrono does not support %Z
    let re = Regex::new(r"notAfter=([A-Za-z]+ +\d+ +[\d:]+ +\d{4}) +[A-Za-z]+\n").unwrap();

    println!();
    println!("SSL Certificates:");
    for (name, path) in config.certs {
        let output = Command::new("openssl")
            .arg("x509")
            .arg("-in")
            .arg(&path)
            .arg("-dates")
            .output()
            .unwrap();
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
