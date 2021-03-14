use serde::Deserialize;
use std::io::Write;
use std::process::Command;
use thiserror::Error;

// TODO: make config better (e.g. curl vs wget, location, format be all seperate options)
#[derive(Debug, Deserialize)]
pub struct WeatherCfg {
    command: Option<String>,
    url: Option<String>,
    loc: Option<String>,
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
    #[error("failed with exit code {exit_code:?}:\n{error:?}")]
    CommandError { exit_code: i32, error: String },

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub fn disp_weather(config: WeatherCfg) -> Result<(), WeatherError> {
    let command = config.command.unwrap_or("curl".to_string());
    let arg = match config.url {
        Some(url) => url,
        None => {
            let mut base = String::from("wttr.in/");
            let loc = config.loc.unwrap_or("".to_owned());
            let loc = loc.replace(", ", ",").replace(" ", "+");
            base.push_str(&loc);
            match config.style.unwrap_or(WeatherStyle::Day) {
                WeatherStyle::Oneline => base.push_str("?format=4"),
                WeatherStyle::Day => base.push_str("?0"),
                WeatherStyle::Full => (),
            }
            base
        }
    };
    let output = Command::new(command).arg(arg).output()?;

    if !output.status.success() {
        return Err(WeatherError::CommandError {
            exit_code: output.status.code().unwrap(),
            error: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    let mut out = std::io::stdout();
    out.write_all(&output.stdout)?;
    out.flush()?;

    Ok(())
}
