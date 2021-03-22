use bytesize::ByteSize;
use itertools::Itertools;
use std::cmp;
use std::collections::HashMap;
use std::iter;
use systemstat::{Filesystem, Platform, System};
use termion::{color, style};
use thiserror::Error;

use crate::constants::INDENT_WIDTH;

#[derive(Error, Debug)]
pub enum FilesystemsError {
    #[error("Empty configuration for filesystems. Please remove the entire block to disable this component.")]
    ConfigEmtpyError,

    #[error("Could not find mount {mount_point:?}")]
    MountNotFoundError { mount_point: String },

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

#[derive(Debug)]
struct Entry<'a> {
    filesystem_name: String,
    dev: &'a str,
    mount_point: &'a str,
    fs_type: &'a str,
    used: String,
    total: String,
    used_ratio: f64,
}

pub type FilesystemsCfg = HashMap<String, String>;

fn parse_into_entry(filesystem_name: String, mount: &Filesystem) -> Entry {
    let total = mount.total.as_u64();
    let avail = mount.avail.as_u64();
    let used = total - avail;

    Entry {
        filesystem_name,
        mount_point: &mount.fs_mounted_on,
        dev: &mount.fs_mounted_from,
        fs_type: &mount.fs_type,
        used: ByteSize::b(used).to_string(),
        total: ByteSize::b(total).to_string(),
        used_ratio: (used as f64) / (total as f64),
    }
}

fn print_row<'a>(items: [&str; 6], column_sizes: impl IntoIterator<Item = &'a usize>) {
    println!(
        "{}",
        items
            .iter()
            .zip(column_sizes.into_iter())
            .map(|(name, size)| format!("{: <size$}", name, size = size))
            .intersperse(" ".repeat(INDENT_WIDTH))
            .collect::<String>()
    );
}

pub fn disp_filesystem(config: FilesystemsCfg, sys: &System) -> Result<(), FilesystemsError> {
    if config.is_empty() {
        return Err(FilesystemsError::ConfigEmtpyError);
    }

    let mounts = sys.mounts()?;
    let mounts: HashMap<String, &Filesystem> = mounts
        .iter()
        .map(|fs| (fs.fs_mounted_on.clone(), fs))
        .collect();

    let entries = config
        .into_iter()
        .map(
            |(filesystem_name, mount_point)| match mounts.get(&mount_point) {
                Some(mount) => Ok(parse_into_entry(filesystem_name, mount)),
                _ => Err(FilesystemsError::MountNotFoundError { mount_point }),
            },
        )
        .collect::<Result<Vec<Entry>, FilesystemsError>>()?;

    let header = ["Filesystems", "Device", "Mount", "Type", "Used", "Total"];

    let column_sizes = entries
        .iter()
        .map(|entry| {
            vec![
                entry.filesystem_name.len() + INDENT_WIDTH,
                entry.dev.len(),
                entry.mount_point.len(),
                entry.fs_type.len(),
                entry.used.len(),
                entry.total.len(),
            ]
        })
        .chain(iter::once(header.iter().map(|x| x.len()).collect()))
        .fold(vec![0; header.len()], |acc, x| {
            x.iter()
                .zip(acc.iter())
                .map(|(a, b)| cmp::max(a, &b).to_owned())
                .collect()
        });

    print_row(header, &column_sizes);

    let bar_width = column_sizes.iter().sum::<usize>() + (header.len() - 2) * INDENT_WIDTH - 2;

    for entry in entries {
        let bar_full = ((bar_width as f64) * entry.used_ratio) as usize;
        let bar_empty = bar_width - bar_full;

        print_row(
            [
                &[" ".repeat(INDENT_WIDTH), entry.filesystem_name].concat(),
                entry.dev,
                entry.mount_point,
                entry.fs_type,
                entry.used.as_str(),
                entry.total.as_str(),
            ],
            &column_sizes,
        );

        println!(
            "{}[{}{}{}{}{}]",
            " ".repeat(INDENT_WIDTH),
            color::Fg(color::Green),
            "=".repeat(bar_full),
            color::Fg(color::LightBlack),
            "=".repeat(bar_empty),
            style::Reset,
        );
    }

    Ok(())
}
