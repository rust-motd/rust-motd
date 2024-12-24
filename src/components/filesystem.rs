use async_trait::async_trait;
use bytesize::ByteSize;
use indexmap::IndexMap;
use itertools::Itertools;
use std::cmp;
use std::iter;
use systemstat::{Filesystem, Platform, System};
use termion::{color, style};
use thiserror::Error;

use crate::component::{Component, Constraints, PrepareReturn};
use crate::config::global_config::GlobalConfig;
use crate::constants::INDENT_WIDTH;
use crate::default_prepare;

const HEADER: [&str; 6] = ["Filesystems", "Device", "Mount", "Type", "Used", "Total"];

/// A container for the mount points specified in the configuration file
#[derive(Clone)]
pub struct Filesystems {
    pub mounts: IndexMap<String, String>,
}

#[async_trait]
impl Component for Filesystems {
    fn prepare(self: Box<Self>, global_config: &GlobalConfig) -> PrepareReturn {
        self.clone()
            .prepare_or_error(global_config)
            .unwrap_or((self, Some(Constraints { min_width: None })))
    }

    // Print should never be called on a raw `Filesystems`
    // Prepare should be called, returning a `PreparedFilesystems`
    async fn print(self: Box<Self>, global_config: &GlobalConfig, width: Option<usize>) {
        let (prepared_filesystems, _) = self.prepare(global_config);
        prepared_filesystems.print(global_config, width).await;
    }
}

/// A prepared, ready-to-print filesystems object
/// This is returned from the prepare phase
/// It is generated based on the user's configuration stored in `Filesystems`
/// and has all the information needed for printing
struct PreparedFilesystems {
    column_sizes: Vec<usize>,
    entries: Vec<Entry>,
    bar_width: usize,
}

#[async_trait]
impl Component for PreparedFilesystems {
    async fn print(self: Box<Self>, global_config: &GlobalConfig, _width: Option<usize>) {
        self.print_or_error(global_config).unwrap_or_else(|err| {
            println!("Filesystem error: {}", err);
        });
        println!();
    }

    default_prepare!();
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

/// Data needed to print one row of the filesystems table
#[derive(Debug)]
struct Entry {
    filesystem_name: String,
    dev: String,
    mount_point: String,
    fs_type: String,
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
        mount_point: mount.fs_mounted_on.to_string(),
        dev: mount.fs_mounted_from.to_string(),
        fs_type: mount.fs_type.to_string(),
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
    pub fn new(mounts: IndexMap<String, String>) -> Self {
        Self { mounts }
    }

    fn prepare_or_error(
        self,
        global_config: &GlobalConfig,
    ) -> Result<PrepareReturn, FilesystemsError> {
        let sys = System::new();

        if self.mounts.is_empty() {
            return Err(FilesystemsError::ConfigEmtpy);
        }

        let mounts = sys.mounts()?;
        let mounts: IndexMap<String, &Filesystem> = mounts
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
            .chain(iter::once(HEADER.iter().map(|x| x.len()).collect()))
            .fold(vec![0; HEADER.len()], |acc, x| {
                x.iter()
                    .zip(acc.iter())
                    .map(|(a, b)| cmp::max(a, b).to_owned())
                    .collect()
            });

        // -2 because "Filesystems" does not count (it is not indented)
        // and because zero indexed
        let bar_width = column_sizes.iter().sum::<usize>() + (HEADER.len() - 2) * INDENT_WIDTH
            - global_config.progress_prefix.len()
            - global_config.progress_suffix.len();
        let fs_display_width =
            bar_width + global_config.progress_prefix.len() + global_config.progress_suffix.len();

        let prepared_filesystems = PreparedFilesystems {
            bar_width,
            column_sizes,
            entries,
        };

        let constraints = Constraints {
            min_width: Some(fs_display_width),
        };

        Ok((Box::new(prepared_filesystems), Some(constraints)))
    }
}

impl PreparedFilesystems {
    fn print_or_error(self, global_config: &GlobalConfig) -> Result<(), FilesystemsError> {
        print_row(HEADER, &self.column_sizes);

        for entry in self.entries {
            let bar_full = ((self.bar_width as f64) * entry.used_ratio) as usize;
            let bar_empty = self.bar_width - bar_full;

            print_row(
                [
                    &[" ".repeat(INDENT_WIDTH), entry.filesystem_name].concat(),
                    &entry.dev[..],
                    &entry.mount_point[..],
                    &entry.fs_type[..],
                    entry.used.as_str(),
                    entry.total.as_str(),
                ],
                &self.column_sizes,
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

        Ok(())
    }
}
