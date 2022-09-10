use async_trait::async_trait;
use bytesize::ByteSize;
use itertools::Itertools;
use std::cmp;
use std::collections::HashMap;
use std::iter;
use systemstat::{Filesystem, Platform, System};
use termion::{color, style};
use thiserror::Error;

use crate::component::Component;
use crate::config::global_config::GlobalConfig;
use crate::constants::INDENT_WIDTH;

pub struct Filesystems {
    pub mounts: HashMap<String, String>,
}

#[async_trait]
impl Component for Filesystems {
    async fn print(self: Box<Self>, global_config: &GlobalConfig) {
        self.print_or_error(global_config).unwrap_or_else(|err| {
            println!("Filesystem error: {}", err);
            None
        });
        println!();
    }
}

#[derive(Error, Debug)]
pub enum FilesystemsError {
    #[error("Empty configuration for filesystems. Please remove the entire block to disable this component.")]
    ConfigEmtpy,

    #[error("Could not find mount {mount_point:?}")]
    MountNotFound { mount_point: String },

    #[error(transparent)]
    IO(#[from] std::io::Error),
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
        Itertools::intersperse(
            items
                .iter()
                .zip(column_sizes.into_iter())
                .map(|(name, size)| format!("{: <size$}", name, size = size)),
            " ".repeat(INDENT_WIDTH)
        )
        .collect::<String>()
    );
}

impl Filesystems {
    pub fn print_or_error(
        self,
        global_config: &GlobalConfig,
    ) -> Result<Option<usize>, FilesystemsError> {
        let sys = System::new();

        if self.mounts.is_empty() {
            return Err(FilesystemsError::ConfigEmtpy);
        }

        let mounts = sys.mounts()?;
        let mounts: HashMap<String, &Filesystem> = mounts
            .iter()
            .map(|fs| (fs.fs_mounted_on.clone(), fs))
            .collect();

        let entries = self
            .mounts
            .into_iter()
            .map(
                |(filesystem_name, mount_point)| match mounts.get(&mount_point) {
                    Some(mount) => Ok(parse_into_entry(filesystem_name, mount)),
                    _ => Err(FilesystemsError::MountNotFound { mount_point }),
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
                    .map(|(a, b)| cmp::max(a, b).to_owned())
                    .collect()
            });

        print_row(header, &column_sizes);

        // -2 because "Filesystems" does not count (it is not indented)
        // and because zero indexed
        let bar_width = column_sizes.iter().sum::<usize>() + (header.len() - 2) * INDENT_WIDTH
            - global_config.progress_prefix.len()
            - global_config.progress_suffix.len();
        let fs_display_width =
            bar_width + global_config.progress_prefix.len() + global_config.progress_suffix.len();

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

            let full_color = match (entry.used_ratio * 100.0) as usize {
                0..=75 => color::Fg(color::Green).to_string(),
                76..=95 => color::Fg(color::Yellow).to_string(),
                _ => color::Fg(color::Red).to_string(),
            };

            println!(
                "{}",
                [
                    " ".repeat(INDENT_WIDTH),
                    global_config.progress_prefix.to_string(),
                    full_color,
                    global_config
                        .progress_full_character
                        .to_string()
                        .repeat(bar_full),
                    color::Fg(color::LightBlack).to_string(),
                    global_config
                        .progress_empty_character
                        .to_string()
                        .repeat(bar_empty),
                    style::Reset.to_string(),
                    global_config.progress_suffix.to_string(),
                ]
                .join("")
            );
        }

        Ok(Some(fs_display_width))
    }
}
