use bytesize::ByteSize;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use systemstat::{Filesystem, Platform, System};
use termion::{color, style};

mod components;
use components::banner::{disp_banner, BannerCfg};
use components::ssl_certs::{disp_ssl, SSLCertsCfg};
use components::uptime::{disp_uptime, UptimeCfg};
mod constants;
use constants::{BAR_WIDTH, INDENT_WIDTH};

#[derive(Debug, Deserialize)]
struct Config {
    banner: Option<BannerCfg>,
    service_status: Option<ServiceStatusCfg>,
    uptime: Option<UptimeCfg>,
    ssl_certificates: Option<SSLCertsCfg>,
    filesystems: Option<FilesystemsCfg>,
    fail_2_ban: Option<Fail2BanCfg>,
    last_login: Option<LastLoginCfg>,
}

type ServiceStatusCfg = HashMap<String, String>;

type FilesystemsCfg = HashMap<String, String>;

#[derive(Debug, Deserialize)]
struct Fail2BanCfg {
    jails: Vec<String>,
}

type LastLoginCfg = HashMap<String, usize>;

fn main() {
    match fs::read_to_string("default_config.toml") {
        Ok(config_str) => {
            let config: Config = toml::from_str(&config_str).unwrap();
            let sys = System::new();

            if let Some(banner_config) = config.banner {
                disp_banner(banner_config);
            }

            if let Some(uptime_config) = config.uptime {
                disp_uptime(uptime_config, &sys)
                    .unwrap_or_else(|err| println!("Uptime error: {}", err));
            }

            if let Some(ssl_certificates_config) = config.ssl_certificates {
                disp_ssl(ssl_certificates_config)
                    .unwrap_or_else(|err| println!("SSL Certificate error: {}", err));
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

fn disp_filesystem(config: FilesystemsCfg) -> Result<(), ()> {
    todo!();
}
