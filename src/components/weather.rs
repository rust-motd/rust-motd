use serde::Deserialize;
use std::io::Write;
use thiserror::Error;
use ureq;

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
    Ureq(#[from] ureq::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

pub fn disp_weather(config: WeatherCfg) -> Result<(), WeatherError> {
    let url = match config.url {
        Some(url) => url,
        None => {
            let mut base = String::from("https://wttr.in/");
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
    let body = ureq::get(&url)
        .set("User-Agent", "curl")
        .call()?
        .into_string()?;

    let mut out = std::io::stdout();
    out.write_all(body.as_bytes())?;
    out.flush()?;

    Ok(())
}
