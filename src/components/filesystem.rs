use async_trait::async_trait;
use bytesize::ByteSize;
use itertools::Itertools;
use std::cmp;
use std::io;
use std::iter;
use systemstat::Filesystem;
use termion::{color, style};
use thiserror::Error;
use unicode_ellipsis::truncate_str;
use unicode_width::UnicodeWidthStr;

use crate::component::{Component, Constraints, PrepareReturn};
use crate::config::global_config::GlobalConfig;
use crate::constants::INDENT_WIDTH;
use crate::default_prepare;

const HEADER: [&str; 6] = ["Filesystems", "Device", "Mount", "Type", "Used", "Total"];

#[derive(Clone, knus::Decode, Debug)]
pub struct Mount {
    #[knus(property)]
    pub name: String,
    #[knus(property)]
    pub mount_point: String,
}

/// A container for the mount points specified in the configuration file
#[derive(Clone, knus::Decode, Debug)]
pub struct Filesystems {
    #[knus(children(name = "filesystem"))]
    pub mounts: Vec<Mount>,
}

#[async_trait]
impl Component for Filesystems {
    fn prepare(self: Box<Self>, global_config: &GlobalConfig) -> PrepareReturn {
        if self.mounts.is_empty() {
            return None;
        }

        let entries: Vec<Result<Entry, FilesystemsError>> = self
            .mounts
            .into_iter()
            .map(|Mount { name, mount_point }| match mount_at(&mount_point) {
                Ok(mount) => Ok(parse_into_entry(name, &mount)),
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    Err(FilesystemsError::MountNotFound { mount_point })
                }
                Err(source) => Err(FilesystemsError::IO {
                    mount_point,
                    source,
                }),
            })
            .collect();
        let column_sizes = entries
            .iter()
            .flatten()
            .map(|entry| {
                vec![
                    entry.filesystem_name.width() + INDENT_WIDTH,
                    entry.dev.width(),
                    entry.mount_point.width(),
                    entry.fs_type.width(),
                    entry.used.width(),
                    entry.total.width(),
                ]
            })
            .chain(iter::once(HEADER.iter().map(|x| x.width()).collect()))
            .fold(vec![0; HEADER.len()], |acc, x| {
                x.iter()
                    .zip(acc.iter())
                    .map(|(a, b)| cmp::max(a, b).to_owned())
                    .collect()
            });

        // -2 because "Filesystems" does not count (it is not indented)
        // and because zero indexed
        let fs_display_width =
            column_sizes.iter().sum::<usize>() + (HEADER.len() - 2) * INDENT_WIDTH;
        let bar_width = fs_display_width.saturating_sub(
            global_config.progress_prefix.width() + global_config.progress_suffix.width(),
        );

        let prepared_filesystems = PreparedFilesystems {
            bar_width,
            column_sizes,
            entries,
        };

        let constraints = Constraints {
            min_width: Some(fs_display_width),
        };

        Some((Box::new(prepared_filesystems), Some(constraints)))
    }

    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        unreachable!("Print should never be called on a raw `Filesystems`. Prepare should be called, returning a `PreparedFilesystems`.");
    }
}

/// A prepared, ready-to-print filesystems object
/// This is returned from the prepare phase
/// It is generated based on the user's configuration stored in `Filesystems`
/// and has all the information needed for printing
struct PreparedFilesystems {
    column_sizes: Vec<usize>,
    entries: Vec<Result<Entry, FilesystemsError>>,
    bar_width: usize,
}

#[async_trait]
impl Component for PreparedFilesystems {
    async fn print(self: Box<Self>, global_config: &GlobalConfig, _width: Option<usize>) {
        self.print_or_error(global_config).unwrap_or_else(|err| {
            println!("Filesystem error: {err}");
        });
        println!();
    }

    default_prepare!();
}

#[derive(Error, Debug)]
pub enum FilesystemsError {
    #[error("Could not find mount {mount_point:?}")]
    MountNotFound { mount_point: String },

    #[error("Could not read mount {mount_point:?}: {source}")]
    IO {
        mount_point: String,
        source: std::io::Error,
    },
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
    let used = total.saturating_sub(mount.free.as_u64());

    Entry {
        filesystem_name,
        mount_point: mount.fs_mounted_on.to_string(),
        dev: truncate_str(&mount.fs_mounted_from, 26).to_string(),
        fs_type: mount.fs_type.to_string(),
        used: ByteSize::b(used).display().si().to_string(),
        total: ByteSize::b(total).display().si().to_string(),
        used_ratio: (used as f64) / ((used + avail) as f64),
    }
}

#[cfg(target_os = "linux")]
fn unescape(field: &str) -> String {
    let mut unescaped = String::with_capacity(field.len());
    let mut rest = field;
    while let Some((head, tail)) = rest.split_once('\\') {
        unescaped.push_str(head);
        match tail.get(..3).map(|code| u8::from_str_radix(code, 8)) {
            Some(Ok(byte)) => {
                unescaped.push(char::from(byte));
                rest = &tail[3..];
            }
            _ => {
                unescaped.push('\\');
                rest = tail;
            }
        }
    }
    unescaped.push_str(rest);
    unescaped
}

#[cfg(target_os = "linux")]
#[allow(clippy::unnecessary_cast)]
fn mount_at(mount_point: &str) -> io::Result<Filesystem> {
    use std::ffi::CString;
    use std::fs;
    use std::mem::MaybeUninit;
    use systemstat::ByteSize;

    let mounts = fs::read("/proc/mounts")?;
    let (fs_mounted_from, fs_type) = String::from_utf8_lossy(&mounts)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace().map(unescape);
            Some((fields.next()?, fields.next()?, fields.next()?))
        })
        .rfind(|(_, target, _)| target == mount_point)
        .map(|(source, _, fs_type)| (source, fs_type))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No such mount"))?;

    let path = CString::new(mount_point)?;
    let mut stat = MaybeUninit::<libc::statvfs>::uninit();
    if unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let stat = unsafe { stat.assume_init() };

    Ok(Filesystem {
        files: (stat.f_files as usize).saturating_sub(stat.f_ffree as usize),
        files_total: stat.f_files as usize,
        files_avail: stat.f_favail as usize,
        free: ByteSize::b(stat.f_bfree as u64 * stat.f_frsize as u64),
        avail: ByteSize::b(stat.f_bavail as u64 * stat.f_frsize as u64),
        total: ByteSize::b(stat.f_blocks as u64 * stat.f_frsize as u64),
        name_max: stat.f_namemax as usize,
        fs_type,
        fs_mounted_from,
        fs_mounted_on: mount_point.to_string(),
    })
}

#[cfg(not(target_os = "linux"))]
fn mount_at(mount_point: &str) -> io::Result<Filesystem> {
    use systemstat::{Platform, System};

    System::new().mount_at(mount_point)
}

fn print_row<'a>(items: [&str; 6], column_sizes: impl IntoIterator<Item = &'a usize>) {
    println!(
        "{}",
        Itertools::intersperse(
            items
                .iter()
                .zip(column_sizes.into_iter())
                .map(|(name, size)| format!("{name}{}", " ".repeat(size - name.width()))),
            " ".repeat(INDENT_WIDTH)
        )
        .collect::<String>()
    );
}

impl Filesystems {
    pub fn new(mounts: Vec<Mount>) -> Self {
        Self { mounts }
    }
}

impl PreparedFilesystems {
    fn print_or_error(self, global_config: &GlobalConfig) -> Result<(), FilesystemsError> {
        print_row(HEADER, &self.column_sizes);

        for entry in self.entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    println!("Filesystem error: {err}");
                    continue;
                }
            };
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
