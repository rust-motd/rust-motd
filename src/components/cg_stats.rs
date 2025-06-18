use lazy_static::lazy_static;
use libc::passwd as c_passwd;
use libc::uid_t;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::mem;
use std::path::{Path, PathBuf};
use std::ptr;
use std::thread::available_parallelism;
use std::time::{Duration, SystemTime, SystemTimeError};

use async_trait::async_trait;
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};
use termion::{color, style};
use thiserror::Error;

use crate::component::{Component, Constraints, PrepareReturn};
use crate::config::global_config::GlobalConfig;
use crate::constants::INDENT_WIDTH;

/// A container for component configuration from the configuration
/// file as well as data prepared for printing.
#[derive(Deserialize)]
pub struct CgStats {
    /// File where to store Cgroup statistic needed by the next run
    state_file: String,
    /// List only Cgroups with higher CPU usage (0.01 ~ 1%)
    threshold: f64,

    #[serde(skip)]
    prepared: Option<PreparedCgStats>,
}

struct PreparedStat {
    name: String,
    load: f64, // CPU load [0, 1]
}

#[derive(Default)]
pub struct PreparedCgStats {
    time_span: Duration,
    max_name_width: usize,
    users: Vec<PreparedStat>,
    services: Vec<PreparedStat>,
}

#[async_trait]
impl Component for CgStats {
    fn prepare(mut self: Box<Self>, global_config: &GlobalConfig) -> PrepareReturn {
        match self.prepare_or_error(global_config) {
            Ok(prepared) => {
                let min_width =
                    INDENT_WIDTH + prepared.max_name_width + "100%".len() + "[=========]".len() + 2 /* spaces */;
                self.prepared = Some(prepared);
                (
                    self,
                    Some(Constraints {
                        min_width: Some(min_width),
                    }),
                )
            }
            Err(e) => {
                eprintln!("Cgroup Statistics error: {e}");
                return (self, None);
            }
        }
    }

    async fn print(self: Box<Self>, global_config: &GlobalConfig, width: Option<usize>) {
        let prepared = if let Some(prepared) = self.prepared {
            prepared
        } else {
            return;
        };
        let secs = prepared.time_span.as_secs();
        let rounded_time = if secs < 180 {
            Duration::from_secs(secs)
        } else {
            Duration::from_secs((secs + 30) / 60 * 60)
        };
        println!(
            "CPU usage in the past {}:{}",
            humantime::format_duration(rounded_time),
            if prepared.users.len() + prepared.services.len() == 0 {
                format!(" {}almost idle{}", color::Fg(color::Green), style::Reset)
            } else {
                "".into()
            }
        );
        let indent = " ".repeat(INDENT_WIDTH);
        let width = width.unwrap_or(global_config.progress_width - INDENT_WIDTH);
        let bar_width = width - INDENT_WIDTH - prepared.max_name_width - 1 - 5;
        for (title, data) in [("Users", &prepared.users), ("Services", &prepared.services)] {
            if !data.is_empty() {
                println!("{indent}{title}:");
            }
            for stat in data {
                println!(
                    "{indent}{indent}{name:<width$} {percent:3.0}% {bar}",
                    name = stat.name,
                    bar = format_bar(global_config, bar_width, stat.load),
                    percent = stat.load * 100.0,
                    width = prepared.max_name_width,
                );
            }
        }
        println!();
    }
}

#[derive(Error, Debug)]
pub enum CgStatsError {
    #[error("File `{0}`: {1}")]
    FileError(PathBuf, io::Error),

    #[error("Failed to parse `{0}`")]
    ParseError(PathBuf),

    #[error("Failed to parse a number in {0}: {1}")]
    ParseIntError(PathBuf, std::num::ParseIntError),

    #[error("Field {1} not found in {0}")]
    MissingField(PathBuf, String),

    #[error(transparent)]
    TomlSerialization(#[from] toml::ser::Error),

    #[error("Failed to calculate time span from time in {0} ({1})")]
    TimeSpan(PathBuf, SystemTimeError),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

impl CgStats {
    pub fn prepare_or_error(
        &self,
        _global_config: &GlobalConfig,
    ) -> Result<PreparedCgStats, CgStatsError> {
        let num_cpus = available_parallelism()?.get();
        let now = read_cg_state()?;

        let mut prepared_cg_stats = PreparedCgStats::default();

        if let Ok(before) = fs::read_to_string(&self.state_file)
            .inspect_err(|e| eprintln!("Reading {} failed: {e}", self.state_file))
            .and_then(|s| {
                toml::from_str::<State>(&s).map_err(|e| {
                    eprintln!("Parsing TOML from {} failed: {e}", self.state_file);
                    io::Error::other(e)
                })
            })
        {
            // Calculate the statistics
            let time_span = now
                .time
                .duration_since(before.time)
                .map_err(|e| CgStatsError::TimeSpan((&self.state_file).into(), e))?;
            let treshold = self.threshold;
            prepared_cg_stats.time_span = time_span;
            prepared_cg_stats.users =
                get_prepared_stats(&now.user, &before.user, time_span, num_cpus, treshold);
            prepared_cg_stats.services =
                get_prepared_stats(&now.system, &before.system, time_span, num_cpus, treshold);
            prepared_cg_stats.max_name_width = prepared_cg_stats
                .users
                .iter()
                .chain(prepared_cg_stats.services.iter())
                .map(|s| s.name.len())
                .max()
                .unwrap_or(0);
        }
        fs::write(&self.state_file, toml::to_string(&now)?)
            .map_err(|e| CgStatsError::FileError(PathBuf::from(&self.state_file), e))?;
        Ok(prepared_cg_stats)
    }
}

/// Statistics read from a single cgroup
#[derive(Serialize, Deserialize)]
struct CgStat {
    usage_usec: u64, // CPU usage
}

/// Statistics from multiple cgroups read at certain time. CPU usage
/// is calculated from two instances of State taken at different
/// times.
#[derive(Serialize, Deserialize)]
struct State {
    time: SystemTime,
    user: HashMap<String, CgStat>,   // user.slice
    system: HashMap<String, CgStat>, // system.slice
}

fn full_color(ratio: f64) -> String {
    match (ratio * 100.) as usize {
        0..=75 => color::Fg(color::Green).to_string(),
        76..=95 => color::Fg(color::Yellow).to_string(),
        _ => color::Fg(color::Red).to_string(),
    }
}

fn format_bar(global_config: &GlobalConfig, width: usize, full_ratio: f64) -> String {
    let without_ends_width =
        width - global_config.progress_suffix.len() - global_config.progress_prefix.len();

    let bar_full = ((without_ends_width as f64) * full_ratio.clamp(0.0, 1.0)).round() as usize;
    let bar_empty = without_ends_width - bar_full;
    let full_color = full_color(full_ratio);

    [
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
}

/// Calculate CPU usage from two states taken at different times. The
/// result will include only Cgroups with CPU usage >= threshold.
fn get_prepared_stats(
    now: &HashMap<String, CgStat>,
    before: &HashMap<String, CgStat>,
    time_span: Duration,
    num_cpus: usize,
    threshold: f64,
) -> Vec<PreparedStat> {
    let mut stats = Vec::new();
    for key in now.keys().sorted() {
        if before.contains_key(key) {
            let s1 = before.get(key).unwrap();
            let s2 = now.get(key).unwrap();
            let load = (s2.usage_usec as i64 - s1.usage_usec as i64) as f64
                / time_span.as_micros() as f64
                / num_cpus as f64;
            if load >= threshold {
                stats.push(PreparedStat {
                    name: key.clone(),
                    load,
                });
            }
        }
    }
    stats
}

/// Read statistics from a single Cgroup
fn read_cg_stat(cg_path: &Path) -> Result<CgStat, CgStatsError> {
    let path = cg_path.join("cpu.stat");
    let f = File::open(path.clone()).map_err(|e| CgStatsError::FileError(path.to_owned(), e))?;
    for line in BufReader::new(f).lines() {
        let l = line?;
        let (key, value) = l
            .split_whitespace()
            .next_tuple()
            .ok_or_else(|| CgStatsError::ParseError(path.clone()))?;
        if let ("usage_usec", val) = (
            key,
            value
                .parse::<u64>()
                .map_err(|e| CgStatsError::ParseIntError(path.clone(), e))?,
        ) {
            return Ok(CgStat { usage_usec: val });
        }
    }
    Err(CgStatsError::MissingField(
        path.clone(),
        "usage_usec".into(),
    ))
}

/// Read statistics from direct children of a Cgroup given by `slice`.
/// The keys of the returned hash map are the names of Cgroups passed
/// through the `rename_key` function.
fn read_stats<F>(slice: &str, rename_key: F) -> Result<HashMap<String, CgStat>, CgStatsError>
where
    F: Fn(&str) -> String,
{
    let dir = ["/sys/fs/cgroup", slice].iter().collect::<PathBuf>();

    let mut stats = HashMap::new();
    for entry in fs::read_dir(dir)? {
        let e = entry?;
        if e.file_type()?.is_dir() {
            let stat = read_cg_stat(&e.path())?;
            stats.insert(rename_key(&e.file_name().to_string_lossy()), stat);
        }
    }
    Ok(stats)
}

// Copied and adapted from https://docs.rs/users/0.11.0/src/users/base.rs.html#326-360
// Copyright (c) 2019 Benjamin Sago, MIT License
fn get_username_by_uid(uid: uid_t) -> Option<CString> {
    let mut passwd = unsafe { mem::zeroed::<c_passwd>() };
    let mut buf = vec![0; 2048];
    let mut result = ptr::null_mut::<c_passwd>();

    loop {
        let r =
            unsafe { libc::getpwuid_r(uid, &mut passwd, buf.as_mut_ptr(), buf.len(), &mut result) };

        if r != libc::ERANGE {
            break;
        }

        let newsize = buf.len().checked_mul(2)?;
        buf.resize(newsize, 0);
    }

    if result.is_null() {
        // There is no such user, or an error has occurred.
        // errno gets set if there’s an error.
        return None;
    }

    if result != &mut passwd {
        // The result of getpwuid_r should be its input passwd.
        return None;
    }
    let ptr = unsafe { result.read() }.pw_name;
    let name = unsafe { CStr::from_ptr(ptr) };
    Some(CString::from(name))
}

lazy_static! {
    static ref SUFFIX_REGEX: Regex = Regex::new(r"\.service|\.scope|\.slice").unwrap();
    static ref UID_REGEX: Regex = Regex::new(r"^user-([0-9]+)\.slice$").unwrap();
}

fn key2username(key: &str) -> Result<String, String> {
    let cap = UID_REGEX.captures(key).ok_or(key.to_owned())?;
    let uid = cap[1].parse::<u32>().map_err(|_| cap[1].to_owned())?;
    let username = get_username_by_uid(uid).ok_or(cap[1].to_owned())?;
    Ok(username.to_string_lossy().to_string())
}

fn read_cg_state() -> Result<State, CgStatsError> {
    let mut state = State {
        time: SystemTime::now(),
        user: HashMap::new(),
        system: HashMap::new(),
    };
    // Read statistics of system services and shorten too long names, e.g.,
    // docker-dcd9a8c71b756de71a4a837c005840f84e0ed92574704ae1c89409c57980aaee.scope
    state.system = read_stats("system.slice", |key| {
        let name_no_suffix = SUFFIX_REGEX.replace(key, "");
        let max_len = 23;
        if name_no_suffix.len() <= max_len {
            name_no_suffix.to_string()
        } else {
            let mut name = name_no_suffix.to_string();
            name.truncate(max_len - 3);
            name += "...";
            name
        }
    })?;

    // Read statistics of users and convert UIDs to user names
    state.user = read_stats("user.slice", |key| match key2username(key) {
        Ok(usename) => usename,
        Err(fallback) => {
            eprint!("warning: Cannot determine user name for {key}");
            fallback
        }
    })?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key2username() {
        // This can fail if UID 0 is not named root
        assert_eq!(key2username("user-0.slice"), Ok(String::from("root")));
        assert_eq!(key2username("xxx"), Err(String::from("xxx")));
        assert_eq!(
            key2username("user-x.slice"),
            Err(String::from("user-x.slice"))
        );
        // This can fail if your system uses UID 4294967286
        assert_eq!(
            key2username("user-4294967286.slice"),
            Err(String::from("4294967286"))
        );
    }
}
