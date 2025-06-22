use async_trait::async_trait;
use serde::Deserialize;
use std::io::Write;
use std::time::Duration;
use thiserror::Error;
use ureq;

use crate::component::Component;
use crate::config::global_config::GlobalConfig;
use crate::default_prepare;

#[derive(Debug, Deserialize, knuffel::Decode)]
pub struct Weather {
    #[knuffel(property)]
    url: Option<String>,
    #[knuffel(property)]
    user_agent: Option<String>,
    #[knuffel(property)]
    proxy: Option<String>,

    #[knuffel(property, default="".into())]
    #[serde(default = "String::new")]
    loc: String,

    #[knuffel(property)]
    style: Option<WeatherStyle>,

    #[knuffel(property, default = 5)]
    #[serde(default = "default_timeout")]
    timeout: u64,
}

fn default_timeout() -> u64 {
    5
}

#[async_trait]
impl Component for Weather {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        self.print_or_error()
            .unwrap_or_else(|err| println!("Weather error: {err}"));
        println!();
    }
    default_prepare!();
}

#[derive(Debug, Deserialize, knuffel::DecodeScalar)]
enum WeatherStyle {
    #[serde(alias = "oneline")]
    Oneline,
    #[serde(alias = "day")]
    Day,
    #[serde(alias = "full")]
    Full,
}

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Empty response body from weather service")]
    ReplyEmpty,

    #[error(transparent)]
    Ureq(#[from] ureq::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

impl Weather {
    pub fn print_or_error(self) -> Result<(), WeatherError> {
        let url = match self.url {
            Some(url) => url,
            None => {
                let mut base = String::from("https://wttr.in/");
                let loc = self.loc.replace(", ", ",").replace(' ', "+");
                base.push_str(&loc);
                match &self.style.as_ref().unwrap_or(&WeatherStyle::Day) {
                    WeatherStyle::Oneline => base.push_str("?format=4"),
                    WeatherStyle::Day => base.push_str("?0"),
                    WeatherStyle::Full => (),
                }
                base
            }
        };

        let proxy = self
            .proxy
            .map_or(Ok(None), |proxy| ureq::Proxy::new(&proxy).map(Some))?;
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(self.timeout)))
            .proxy(proxy)
            .build();
        let agent = ureq::Agent::new_with_config(config);

        let user_agent = match self.user_agent {
            Some(user_agent) => user_agent,
            None => String::from("curl"),
        };

        let body = agent
            .get(&url)
            .header("User-Agent", &user_agent)
            .call()?
            .body_mut()
            .read_to_string()?;

        let mut body = body.lines();
        let first_line = body
            .next()
            .ok_or(WeatherError::ReplyEmpty)?
            .replace('+', " ") // de-slugify the placename by removing '+'
            .replace(',', ", ") // and adding a space after commas
            .replace("  ", " "); // necessary because sometimes there are already spaces
                                 // after the comma in the placename
        let body = body
            .map(|x| [x, "\n"].concat())
            .collect::<Vec<String>>()
            .join("");

        let mut out = std::io::stdout();
        out.write_all(&[first_line.as_bytes(), "\n".as_bytes()].concat())?;
        out.write_all(body.as_bytes())?;
        out.flush()?;

        Ok(())
    }
}
