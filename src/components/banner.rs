use async_trait::async_trait;
use serde::Deserialize;
use termion::{color, style};
use thiserror::Error;

use crate::command::{BetterCommand, BetterCommandError};
use crate::component::Component;
use crate::config::global_config::GlobalConfig;
use crate::default_prepare;

#[derive(Debug, Deserialize)]
pub struct Banner {
    color: BannerColor,
    command: String,
}

#[async_trait]
impl Component for Banner {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        self.print_or_error()
            .unwrap_or_else(|err| println!("Banner error: {}", err));
        println!();
    }
    default_prepare!();
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BannerColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    LightBlack,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    LightWhite,
}

#[derive(Error, Debug)]
pub enum BannerError {
    #[error(transparent)]
    BetterCommandError(#[from] BetterCommandError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

impl Banner {
    pub fn print_or_error(self) -> Result<(), BannerError> {
        // We probably don't have to handle command not found for sh
        let output = BetterCommand::new("sh")
            .arg("-c")
            .arg(&self.command)
            .check_status_and_get_output_string()?;

        let banner_color = match self.color {
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
}
