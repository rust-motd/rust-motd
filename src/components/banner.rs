use serde::Deserialize;
use std::process::Command;
use termion::{color, style};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct BannerCfg {
    color: String,
    command: String,
}

#[derive(Error, Debug)]
pub enum BannerError {
    // TODO: The executable should be configurable too
    #[error("failed with exit code {exit_code:?}:\n{error:?}")]
    CommandError { exit_code: i32, error: String },

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub fn disp_banner(config: BannerCfg) -> Result<(), BannerError> {
    // TODO: Make colour configurable
    // We probably don't have to handle command not found for sh
    let output = Command::new("sh").arg("-c").arg(config.command).output()?;

    if !output.status.success() {
        return Err(BannerError::CommandError {
            exit_code: output.status.code().unwrap(),
            error: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    println!(
        "{}{}{}",
        color::Fg(color::Red),
        &String::from_utf8_lossy(&output.stdout).trim_end(),
        style::Reset
    );

    Ok(())
}
