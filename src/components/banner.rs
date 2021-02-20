use serde::Deserialize;
use std::process::Command;
use termion::{color, style};

#[derive(Debug, Deserialize)]
pub struct BannerCfg {
    color: String,
    command: String,
}

pub fn disp_banner(config: BannerCfg) {
    // TODO: Make colour configurable
    let output = Command::new("sh")
        .arg("-c")
        .arg(config.command)
        .output()
        .unwrap()
        .stdout;
    println!(
        "{}{}{}",
        color::Fg(color::Red),
        &String::from_utf8_lossy(&output),
        style::Reset
    );
}
