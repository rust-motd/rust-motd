use bytesize::ByteSize;
use chrono::{Duration, TimeZone, Utc};
use humantime::format_duration;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use systemstat::{Filesystem, Platform, System};
use termion::{color, style};

// TODO: Move config to it's own file
#[derive(Debug, Deserialize)]
struct Config {
    banner: Option<Banner>,
    service_status: Option<HashMap<String, String>>,
    uptime: Option<Uptime>,
    ssl_certificates: Option<SSLCerts>,
    filesystems: Option<HashMap<String, String>>,
    fail_2_ban: Option<Fail2Ban>,
    last_login: Option<HashMap<String, usize>>,
}

#[derive(Debug, Deserialize)]
struct Banner {
    color: String,
    command: String,
}

#[derive(Debug, Deserialize)]
struct Uptime {
    prefix: String,
}

#[derive(Debug, Deserialize)]
struct SSLCerts {
    sort_method: String, // TODO: Maybe switch to enum insead of string
    // need to figure out how to do this in Serde
    certs: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct Fail2Ban {
    jails: Vec<String>,
}

const LINE_WIDTH: u64 = 60;
const INDENT_WIDTH: u64 = 2;
const BAR_WIDTH: u64 = LINE_WIDTH - INDENT_WIDTH - 2;

fn main() {
    match fs::read_to_string("default_config.toml") {
        Ok(config_str) => {
            let config: Config = toml::from_str(&config_str).unwrap();
            let sys = System::new();

            // TODO: Make colour configurable
            if let Some(banner) = config.banner {
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(banner.command)
                    .output()
                    .unwrap()
                    .stdout;
                println!(
                    "{}{}{}",
                    color::Fg(color::Red),
                    &String::from_utf8_lossy(&output),
                    style::Reset
                );
            }

            if let Some(uptime_config) = config.uptime {
                match sys.uptime() {
                    Ok(uptime) => {
                        println!(
                            "{} {}",
                            uptime_config.prefix,
                            format_duration(uptime).to_string()
                        )
                    }
                    Err(x) => println!("Uptime error: {}", x),
                }
            }

            // TODO: Support time zone
            // chrono does not support %Z
            if let Some(ssl_certificates) = config.ssl_certificates {
                let re =
                    Regex::new(r"notAfter=([A-Za-z]+ +\d+ +[\d:]+ +\d{4}) +[A-Za-z]+\n").unwrap();

                println!();
                println!("SSL Certificates");
                for (name, path) in ssl_certificates.certs {
                    let output = Command::new("openssl")
                        .arg("x509")
                        .arg("-in")
                        .arg(&path)
                        .arg("-dates")
                        .output()
                        .unwrap();
                    let output = String::from_utf8_lossy(&output.stdout);
                    match re.captures(&output) {
                        Some(captures) => match Utc.datetime_from_str(&captures[1], "%B %_d %T %Y")
                        {
                            Ok(date) => {
                                let now = Utc::now();
                                let status = if date < now {
                                    format!("{}expired on{}", color::Fg(color::Red), style::Reset)
                                } else if date < now + Duration::days(30) {
                                    format!(
                                        "{}expiring on{}",
                                        color::Fg(color::Yellow),
                                        style::Reset
                                    )
                                } else {
                                    format!(
                                        "{}valid until{}",
                                        color::Fg(color::Green),
                                        style::Reset
                                    )
                                };
                                println!(
                                    "{}{} {} {}",
                                    " ".repeat(INDENT_WIDTH as usize),
                                    name,
                                    status,
                                    date
                                );
                            }
                            Err(x) => println!("{}", x),
                        },
                        None => println!(
                            "{}Error parsing certificate {}",
                            " ".repeat(INDENT_WIDTH as usize),
                            name
                        ),
                    }
                }
            }

            if let Some(filesystems) = config.filesystems {
                match sys.mounts() {
                    Ok(mounts) => {
                        let mounts: HashMap<String, &Filesystem> = mounts
                            .iter()
                            .map(|fs| (fs.fs_mounted_on.clone(), fs))
                            .collect();

                        println!();
                        println!("Filsystems");
                        for (filesystem_name, mount_point) in filesystems {
                            match mounts.get(&mount_point) {
                                Some(mount) => {
                                    let total = mount.total.as_u64();
                                    let avail = mount.avail.as_u64();
                                    let used = total - avail;
                                    let bar_full = BAR_WIDTH * used / total;
                                    let bar_empty = BAR_WIDTH - bar_full;

                                    println!(
                                        "{}{} {} -> {} ({}) {}/{}",
                                        " ".repeat(INDENT_WIDTH as usize),
                                        filesystem_name,
                                        mount.fs_mounted_from,
                                        mount.fs_mounted_on,
                                        mount.fs_type,
                                        ByteSize::b(used),
                                        ByteSize::b(total)
                                    );
                                    println!(
                                        "{}[{}{}{}{}{}]",
                                        " ".repeat(INDENT_WIDTH as usize),
                                        color::Fg(color::Green),
                                        "=".repeat(bar_full as usize),
                                        color::Fg(color::LightBlack),
                                        "=".repeat(bar_empty as usize),
                                        style::Reset,
                                    );
                                }
                                None => println!("Could not find mount {}", mount_point),
                            }
                        }
                    }
                    Err(e) => println!("Error reading mounts: {}", e),
                }
            }
        }
        Err(e) => println!("Error reading config file: {}", e),
    }
}
