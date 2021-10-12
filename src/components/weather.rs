use serde::Deserialize;
use std::io::Write;
use thiserror::Error;

use crate::command::{BetterCommand, BetterCommandError};

// TODO: make config better (e.g. curl vs wget, location, format be all seperate options)
#[derive(Debug, Deserialize)]
pub struct WeatherCfg {
    command: Option<String>,
    url: Option<String>,

    #[serde(default = "String::new")]
    loc: String,

    style: Option<WeatherStyle>,
}

#[derive(Debug, Deserialize)]
enum WeatherStyle {
    #[serde(alias = "oneline")]
    Oneline,
    #[serde(alias = "day")]
    Day,
    #[serde(alias = "full")]
    Full,
}

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error(transparent)]
    BetterCommandError(#[from] BetterCommandError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub fn disp_weather(config: WeatherCfg) -> Result<(), WeatherError> {
    let command = config.command.unwrap_or_else(|| "curl".to_string());
    let arg = match config.url {
        Some(url) => url,
        None => {
            let mut base = String::from("wttr.in/");
            let loc = config.loc.replace(", ", ",").replace(" ", "+");
            base.push_str(&loc);
            match config.style.unwrap_or(WeatherStyle::Day) {
                WeatherStyle::Oneline => base.push_str("?format=4"),
                WeatherStyle::Day => base.push_str("?0"),
                WeatherStyle::Full => (),
            }
            base
        }
    };
    let output = BetterCommand::new(&command[..]).arg(arg).output()?;

    let mut out = std::io::stdout();
    out.write_all(&output.stdout)?;
    out.flush()?;

    Ok(())
}
