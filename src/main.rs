use bytesize::ByteSize;
use figlet_rs::FIGfont;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use systemstat::{Filesystem, Platform, System};
use termion::{color, style};

// TODO: Move config to it's own file
#[derive(Debug, Deserialize)]
struct Config {
    ascii_text_art: Option<Ata>,
    service_status: Option<HashMap<String, String>>,
    uptime: Option<Uptime>,
    ssl_certificates: Option<SSLCerts>,
    filesystems: Option<HashMap<String, String>>,
    fail_2_ban: Option<Fail2Ban>,
    last_login: Option<HashMap<String, usize>>,
}

#[derive(Debug, Deserialize)]
struct Ata {
    font: String,
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
const BAR_WIDTH: u64 = LINE_WIDTH - 2;

fn main() {
    match fs::read_to_string("default_config.toml") {
        Ok(config_str) => {
            let config: Config = toml::from_str(&config_str).unwrap();
            let sys = System::new();

            // TODO: Fix space between characters
            // https://github.com/yuanbohan/rs-figlet/issues/9
            if let Some(ascii_text_art) = config.ascii_text_art {
                let slant_font = FIGfont::from_file("resources/slant.flf").unwrap();
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(ascii_text_art.command)
                    .output()
                    .unwrap()
                    .stdout;
                let figure = slant_font.convert(&String::from_utf8_lossy(&output));
                println!(
                    "{}{}{}",
                    color::Fg(color::Red),
                    figure.unwrap(),
                    style::Reset
                );
            }

            if let Some(filesystems) = config.filesystems {
                match sys.mounts() {
                    Ok(mounts) => {
                        let mounts: HashMap<String, &Filesystem> = mounts
                            .iter()
                            .map(|fs| (fs.fs_mounted_on.clone(), fs))
                            .collect();

                        for (filesystem_name, mount_point) in filesystems {
                            match mounts.get(&mount_point) {
                                Some(mount) => {
                                    let total = mount.total.as_u64();
                                    let avail = mount.avail.as_u64();
                                    let used = total - avail;
                                    let bar_full = BAR_WIDTH * used / total;
                                    let bar_empty = BAR_WIDTH - bar_full;

                                    println!(
                                        "{} {} -> {} ({}) {}/{}",
                                        filesystem_name,
                                        mount.fs_mounted_from,
                                        mount.fs_mounted_on,
                                        mount.fs_type,
                                        ByteSize::b(used),
                                        ByteSize::b(total)
                                    );
                                    println!(
                                        "[{}{}{}{}{}]",
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
