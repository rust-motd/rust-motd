use async_trait::async_trait;
use serde::Deserialize;
use termion::{color, style};
use thiserror::Error;

use crate::command::{BetterCommand, BetterCommandError};
use crate::component::Component;
use crate::config::global_config::GlobalConfig;
use crate::default_prepare;

#[derive(knus::Decode, Debug, Deserialize)]
pub struct Command {
    #[knus(property, default=Color::White)]
    color: Color,
    #[knus(argument)]
    command: String,
}

#[async_trait]
impl Component for Command {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        self.print_or_error()
            .unwrap_or_else(|err| println!("Command error: {err}"));
        println!();
    }
    default_prepare!();
}

#[derive(knus::DecodeScalar, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Color {
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
pub enum CommandError {
    #[error(transparent)]
    BetterCommandError(#[from] BetterCommandError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

impl Command {
    pub fn print_or_error(self) -> Result<(), CommandError> {
        // We probably don't have to handle command not found for sh
        let output = BetterCommand::new("sh")
            .arg("-c")
            .arg(&self.command)
            .check_status_and_get_output_string()?;

        let color = match self.color {
            Color::Black => color::Black.fg_str(),
            Color::Red => color::Red.fg_str(),
            Color::Yellow => color::Yellow.fg_str(),
            Color::Green => color::Green.fg_str(),
            Color::Blue => color::Blue.fg_str(),
            Color::Magenta => color::Magenta.fg_str(),
            Color::Cyan => color::Cyan.fg_str(),
            Color::White => color::White.fg_str(),
            Color::LightBlack => color::LightBlack.fg_str(),
            Color::LightRed => color::LightRed.fg_str(),
            Color::LightYellow => color::LightYellow.fg_str(),
            Color::LightGreen => color::LightGreen.fg_str(),
            Color::LightBlue => color::LightBlue.fg_str(),
            Color::LightMagenta => color::LightMagenta.fg_str(),
            Color::LightCyan => color::LightCyan.fg_str(),
            Color::LightWhite => color::LightWhite.fg_str(),
        };

        println!("{}{}{}", color, &output.trim_end(), style::Reset);

        Ok(())
    }
}
