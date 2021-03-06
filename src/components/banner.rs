use serde::Deserialize;
use termion::{color, style};
use thiserror::Error;

use crate::command::{BetterCommand, BetterCommandError};

#[derive(Debug, Deserialize)]
pub struct BannerCfg {
    color: BannerColor,
    command: String,
}

#[derive(Debug, Deserialize)]
enum BannerColor {
    #[serde(alias = "black")]
    Black,
    #[serde(alias = "red")]
    Red,
    #[serde(alias = "green")]
    Green,
    #[serde(alias = "yellow")]
    Yellow,
    #[serde(alias = "blue")]
    Blue,
    #[serde(alias = "magenta")]
    Magenta,
    #[serde(alias = "cyan")]
    Cyan,
    #[serde(alias = "white")]
    White,
    #[serde(alias = "light_black")]
    LightBlack,
    #[serde(alias = "light_red")]
    LightRed,
    #[serde(alias = "light_green")]
    LightGreen,
    #[serde(alias = "light_yellow")]
    LightYellow,
    #[serde(alias = "light_blue")]
    LightBlue,
    #[serde(alias = "light_magenta")]
    LightMagenta,
    #[serde(alias = "light_cyan")]
    LightCyan,
    #[serde(alias = "light_white")]
    LightWhite,
}

#[derive(Error, Debug)]
pub enum BannerError {
    #[error(transparent)]
    BetterCommandError(#[from] BetterCommandError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub fn disp_banner(config: BannerCfg) -> Result<(), BannerError> {
    // We probably don't have to handle command not found for sh
    let output = BetterCommand::new("sh")
        .arg("-c")
        .arg(config.command)
        .check_status_and_get_output_string()?;

    let banner_color = match config.color {
        BannerColor::Black => color::Black.fg_str(),
        BannerColor::Red => color::Red.fg_str(),
        BannerColor::Yellow => color::Yellow.fg_str(),
        BannerColor::Green => color::Green.fg_str(),
        BannerColor::Blue => color::Blue.fg_str(),
        BannerColor::Magenta => color::Magenta.fg_str(),
        BannerColor::Cyan => color::Cyan.fg_str(),
        BannerColor::White => color::White.fg_str(),
        BannerColor::LightBlack => color::LightBlack.fg_str(),
        BannerColor::LightRed => color::LightRed.fg_str(),
        BannerColor::LightYellow => color::LightYellow.fg_str(),
        BannerColor::LightGreen => color::LightGreen.fg_str(),
        BannerColor::LightBlue => color::LightBlue.fg_str(),
        BannerColor::LightMagenta => color::LightMagenta.fg_str(),
        BannerColor::LightCyan => color::LightCyan.fg_str(),
        BannerColor::LightWhite => color::LightWhite.fg_str(),
    };

    println!("{}{}{}", banner_color, &output.trim_end(), style::Reset);

    Ok(())
}
