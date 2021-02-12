use bytesize::ByteSize;
use itertools::Itertools;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::path::Path;
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

            let mounts = sys.mounts().unwrap();
            let mounts: HashMap<String, &Filesystem> = mounts
                .iter()
                .map(|fs| (fs.fs_mounted_on.clone(), fs))
                .collect();

            println!("{:?}", mounts);

            /*
            let mounts: HashMap<&str, &Filesystem> = mounts
                .iter()
                .group_by(|x| &x.into_iter().nth(0).unwrap().fs_mounted_on[..])
                .into_iter()
                .map(|(k, mut v)| (k, v.next().unwrap().into_iter().nth(0).unwrap()))
                .collect();
            println!("{:?}", config);

            for (filesystem_name, mount_point) in config.filesystems.unwrap().iter() {
                println!("{}: {}", filesystem_name, mount_point);
            }
            */
        }
        Err(e) => println!("Error reading config file: {}", e),
    }

    // match sys.mounts() {
    //     Ok(mounts) => {
    //         for mount in mounts.iter() {
    //         }
    //     }
    //     Err(x) => println!("\nMounts: error: {}", x),
    // }

    // match sys.mount_at(Path::new("/")) {
    //     Ok(mount) => {
    //         let total = mount.total.as_u64();
    //         let avail = mount.avail.as_u64();
    //         let used = total - avail;
    //         let bar_full = BAR_WIDTH * used / total;
    //         let bar_empty = BAR_WIDTH - bar_full;

    //         println!(
    //             "{} -> {} ({}) {}/{}",
    //             mount.fs_mounted_from,
    //             mount.fs_mounted_on,
    //             mount.fs_type,
    //             ByteSize::b(used),
    //             ByteSize::b(total)
    //         );
    //         println!(
    //             "[{}{}{}{}{}]",
    //             color::Fg(color::Green),
    //             "=".repeat(bar_full as usize),
    //             color::Fg(color::LightBlack),
    //             "=".repeat(bar_empty as usize),
    //             style::Reset,
    //         );
    //     }
    // }
}
