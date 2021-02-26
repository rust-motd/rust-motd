use serde::Deserialize;
use std::io::Write;
use std::process::Command;
use thiserror::Error;

// TODO: make config better (e.g. curl vs wget, location, format be all seperate options)
#[derive(Debug, Deserialize)]
pub struct WeatherCfg {
    url: String,
}

#[derive(Error, Debug)]
pub enum WeatherError {
    // TODO: The executable should be configurable too
    #[error("failed with exit code {exit_code:?}:\n{error:?}")]
    CommandError { exit_code: i32, error: String },

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub fn disp_weather(config: WeatherCfg) -> Result<(), WeatherError> {
    let executable = "curl";
    let output = Command::new(executable).arg(config.url).output()?;

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
