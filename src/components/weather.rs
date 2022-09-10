use async_trait::async_trait;
use serde::Deserialize;
use std::io::Write;
use thiserror::Error;
use ureq;

use crate::component::{Component, Constraints};
use crate::config::global_config::GlobalConfig;

#[derive(Debug, Deserialize)]
pub struct Weather {
    url: Option<String>,

    proxy: Option<String>,

    #[serde(default = "String::new")]
    loc: String,

    style: Option<WeatherStyle>,
}

#[async_trait]
impl Component for Weather {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        self.print_or_error()
            .unwrap_or_else(|err| println!("Weather error: {}", err));
        println!();
    }
    fn prepare(
        self: Box<Self>,
        _global_config: &GlobalConfig,
    ) -> (Box<dyn Component>, Option<Constraints>) {
        (self, None)
    }
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

        let agent = match self.proxy {
            Some(proxy) => {
                let proxy = ureq::Proxy::new(proxy)?;
                ureq::AgentBuilder::new().proxy(proxy).build()
            }
            None => ureq::AgentBuilder::new().build(),
        };
        let body = agent
            .get(&url)
            .set("User-Agent", "curl")
            .call()?
            .into_string()?;

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
