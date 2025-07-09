use async_trait::async_trait;
use serde::Deserialize;
use systemstat::{saturating_sub_bytes, Platform, System};
use termion::{color, style};
use thiserror::Error;

use crate::component::Component;
use crate::config::global_config::GlobalConfig;
use crate::constants::INDENT_WIDTH;
use crate::default_prepare;

#[derive(knus::Decode, Debug, Deserialize)]
pub struct Memory {
    #[knus(property)]
    swap_pos: SwapPosition,
}

#[async_trait]
impl Component for Memory {
    async fn print(self: Box<Self>, global_config: &GlobalConfig, width: Option<usize>) {
        self.print_or_error(global_config, width)
            .unwrap_or_else(|err| println!("Memory error: {err}"));
        println!();
    }
    default_prepare!();
}

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Could not find memory quantity {quantity:?}")]
    MemoryNotFound { quantity: String },

    #[allow(dead_code)]
    #[error("Getting memory information is not supported on the current platform (see issue #20)")]
    UnsupportedPlatform,

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

#[derive(knus::DecodeScalar, Debug, Deserialize, PartialEq, Default)]
enum SwapPosition {
    #[default]
    #[serde(alias = "beside")]
    Beside,
    #[serde(alias = "below")]
    Below,
    #[serde(alias = "none")]
    None,
}

struct MemoryUsage {
    name: String,
    used: String,
    total: String,
    used_ratio: f64,
}

impl MemoryUsage {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    fn get_by_name(
        name: String,
        sys: &System,
        free_name: &str,
        total_name: &str,
    ) -> Result<Self, MemoryError> {
        let memory = sys.memory()?;
        let total =
            memory
                .platform_memory
                .meminfo
                .get(total_name)
                .ok_or(MemoryError::MemoryNotFound {
                    quantity: total_name.to_string(),
                })?;

        let free =
            memory
                .platform_memory
                .meminfo
                .get(free_name)
                .ok_or(MemoryError::MemoryNotFound {
                    quantity: free_name.to_string(),
                })?;
        let used = saturating_sub_bytes(*total, *free);
        Ok(MemoryUsage {
            name,
            used: used.to_string(),
            total: total.to_string(),
            used_ratio: used.as_u64() as f64 / total.as_u64() as f64,
        })
    }

    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    fn get_by_name(
        name: String,
        sys: &System,
        free_name: &str,
        total_name: &str,
    ) -> Result<Self, MemoryError> {
        Err(MemoryError::UnsupportedPlatform)
    }
}

fn format_bar(
    global_config: &GlobalConfig,
    width: usize,
    full_ratio: f64,
    full_color: String,
) -> String {
    let without_ends_width =
        width - global_config.progress_suffix.len() - global_config.progress_prefix.len();

    let bar_full = ((without_ends_width as f64) * full_ratio) as usize;
    let bar_empty = without_ends_width - bar_full;

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

fn full_color(ratio: f64) -> String {
    match (ratio * 100.) as usize {
        0..=75 => color::Fg(color::Green).to_string(),
        76..=95 => color::Fg(color::Yellow).to_string(),
        _ => color::Fg(color::Red).to_string(),
    }
}

fn print_stacked(entries: Vec<MemoryUsage>, width: usize, global_config: &GlobalConfig) {
    for entry in entries {
        println!(
            "{}{}: {} / {}",
            " ".repeat(INDENT_WIDTH),
            entry.name,
            entry.used,
            entry.total
        );
        let full_color = full_color(entry.used_ratio);
        let bar = format_bar(global_config, width, entry.used_ratio, full_color);
        println!(
            "{indent}{bar}",
            indent = " ".repeat(INDENT_WIDTH),
            bar = bar
        );
    }
}

impl Memory {
    pub fn print_or_error(
        self,
        global_config: &GlobalConfig,
        width: Option<usize>,
    ) -> Result<(), MemoryError> {
        let sys = System::new();
        let width = width.unwrap_or(global_config.progress_width - INDENT_WIDTH);

        let ram_usage =
            MemoryUsage::get_by_name("RAM".to_string(), &sys, "MemAvailable", "MemTotal")?;
        println!("Memory");
        match self.swap_pos {
            SwapPosition::None => print_stacked(vec![ram_usage], width, global_config),
            SwapPosition::Below => {
                let swap_usage =
                    MemoryUsage::get_by_name("Swap".to_string(), &sys, "SwapFree", "SwapTotal")?;
                print_stacked(vec![ram_usage, swap_usage], width, global_config)
            }
            SwapPosition::Beside => {
                let swap_usage =
                    MemoryUsage::get_by_name("Swap".to_string(), &sys, "SwapFree", "SwapTotal")?;

                let min_spacing = 1;
                let bar_width = (width - min_spacing) / 2;
                let spacing = width - 2 * bar_width;
                let spacing = " ".repeat(spacing);

                let ram_label = format!(
                    "{}: {} / {}",
                    ram_usage.name, ram_usage.used, ram_usage.total
                );
                let swap_label = format!(
                    "{}: {} / {}",
                    swap_usage.name, swap_usage.used, swap_usage.total
                );
                println!(
                    "{}{ram_label:padding$}{spacing}{swap_label}",
                    " ".repeat(INDENT_WIDTH),
                    ram_label = ram_label,
                    padding = bar_width,
                    spacing = spacing,
                    swap_label = swap_label
                );
                let bar_color = full_color(ram_usage.used_ratio);
                let ram_bar = format_bar(global_config, bar_width, ram_usage.used_ratio, bar_color);

                let bar_color = full_color(swap_usage.used_ratio);
                let swap_bar =
                    format_bar(global_config, bar_width, swap_usage.used_ratio, bar_color);
                println!(
                    "{indent}{ram}{spacing}{swap}",
                    indent = " ".repeat(INDENT_WIDTH),
                    ram = ram_bar,
                    spacing = spacing,
                    swap = swap_bar
                );
            }
        }

        Ok(())
    }
}
