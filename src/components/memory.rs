use serde::Deserialize;
use systemstat::{saturating_sub_bytes, Platform, System};
use termion::{color, style};
use thiserror::Error;

use crate::constants::{GlobalSettings, INDENT_WIDTH};

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Could not find memory quantity {quantity:?}")]
    MemoryNotFound { quantity: String },

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

#[derive(Debug, Deserialize)]
pub struct MemoryCfg {
    swap_pos: SwapPosition,
}

#[derive(Debug, Deserialize, PartialEq)]
enum SwapPosition {
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
}

fn print_bar(
    global_settings: &GlobalSettings,
    full_color: String,
    bar_full: usize,
    bar_empty: usize,
) {
    print!(
        "{}",
        [
            " ".repeat(INDENT_WIDTH),
            global_settings.progress_prefix.to_string(),
            full_color,
            global_settings
                .progress_full_character
                .to_string()
                .repeat(bar_full),
            color::Fg(color::LightBlack).to_string(),
            global_settings
                .progress_empty_character
                .to_string()
                .repeat(bar_empty),
            style::Reset.to_string(),
            global_settings.progress_suffix.to_string(),
        ]
        .join("")
    );
}

fn full_color(ratio: f64) -> String {
    match (ratio * 100.) as usize {
        0..=75 => color::Fg(color::Green).to_string(),
        76..=95 => color::Fg(color::Yellow).to_string(),
        _ => color::Fg(color::Red).to_string(),
    }
}

pub fn disp_memory(
    config: MemoryCfg,
    global_settings: &GlobalSettings,
    sys: &System,
    size_hint: Option<usize>,
) -> Result<(), MemoryError> {
    let size_hint = size_hint.unwrap_or(global_settings.progress_width - INDENT_WIDTH);

    let ram_usage = MemoryUsage::get_by_name("RAM".to_string(), sys, "MemAvailable", "MemTotal")?;
    println!("Memory");
    match config.swap_pos {
        SwapPosition::Below => {
            let swap_usage =
                MemoryUsage::get_by_name("Swap".to_string(), sys, "SwapFree", "SwapTotal")?;
            let entries = vec![ram_usage, swap_usage];
            let bar_width = size_hint
                - global_settings.progress_prefix.len()
                - global_settings.progress_suffix.len();
            for entry in entries {
                let full_color = full_color(entry.used_ratio);
                let bar_full = ((bar_width as f64) * entry.used_ratio) as usize;
                let bar_empty = bar_width - bar_full;
                println!(
                    "{}{}: {} / {}",
                    " ".repeat(INDENT_WIDTH),
                    entry.name,
                    entry.used,
                    entry.total
                );
                print_bar(global_settings, full_color, bar_full, bar_empty);
                println!();
            }
        }
        SwapPosition::Beside => {
            let swap_usage =
                MemoryUsage::get_by_name("Swap".to_string(), sys, "SwapFree", "SwapTotal")?;

            let bar_width = ((size_hint
                - global_settings.progress_prefix.len()
                - global_settings.progress_suffix.len())
                / 2)
                - INDENT_WIDTH;

            let ram_label = format!(
                "{}: {} / {}",
                ram_usage.name, ram_usage.used, ram_usage.total
            );
            let swap_label = format!(
                "{}: {} / {}",
                swap_usage.name, swap_usage.used, swap_usage.total
            );
            println!(
                "{}{}{}{}",
                " ".repeat(INDENT_WIDTH),
                ram_label,
                " ".repeat(bar_width - ram_label.len() + 2 * INDENT_WIDTH),
                swap_label
            );
            let bar_color = full_color(ram_usage.used_ratio);
            let bar_full = ((bar_width as f64) * ram_usage.used_ratio) as usize;
            let bar_empty = bar_width - bar_full;
            print_bar(global_settings, bar_color, bar_full, bar_empty);

            let bar_color = full_color(swap_usage.used_ratio);
            let bar_full = ((bar_width as f64) * swap_usage.used_ratio) as usize;
            let bar_empty = bar_width - bar_full;
            print_bar(global_settings, bar_color, bar_full, bar_empty);
            println!();
        }
        SwapPosition::None => {
            let bar_width = size_hint
                - global_settings.progress_prefix.len()
                - global_settings.progress_suffix.len();
            let full_color = full_color(ram_usage.used_ratio);
            let bar_full = ((bar_width as f64) * ram_usage.used_ratio) as usize;
            let bar_empty = bar_width - bar_full;
            println!(
                "{}{}: {} / {}",
                " ".repeat(INDENT_WIDTH),
                ram_usage.name,
                ram_usage.used,
                ram_usage.total
            );
            print_bar(global_settings, full_color, bar_full, bar_empty);
            println!();
        }
    }

    Ok(())
}
