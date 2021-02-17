use bytesize::ByteSize;
use std::collections::HashMap;
use systemstat::{Filesystem, Platform, System};
use termion::{color, style};

use crate::constants::{BAR_WIDTH, INDENT_WIDTH};

pub type FilesystemsCfg = HashMap<String, String>;

pub fn disp_filesystem(config: FilesystemsCfg, sys: &System) -> Result<(), std::io::Error> {
    match sys.mounts() {
        Ok(mounts) => {
            let mounts: HashMap<String, &Filesystem> = mounts
                .iter()
                .map(|fs| (fs.fs_mounted_on.clone(), fs))
                .collect();

            println!();
            println!("Filsystems");
            for (filesystem_name, mount_point) in config {
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
    Ok(())
}
