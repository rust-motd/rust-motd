use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::thread::available_parallelism;

use async_trait::async_trait;
use interpolator::Formattable;
use serde::Deserialize;
use systemstat::{Platform, System};
use termion::{color, style};

use crate::component::Component;
use crate::config::global_config::GlobalConfig;
use crate::default_prepare;

#[derive(knuffel::Decode, Debug, Deserialize)]
pub struct LoadAvg {
    #[knuffel(property)]
    format: String,
    #[knuffel(property)]
    warn_threshold: Option<f32>,
    #[knuffel(property)]
    bad_threshold: Option<f32>,
}

#[async_trait]
impl Component for LoadAvg {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        self.print_or_error()
            .unwrap_or_else(|err| println!("LoadAvg error: {err}"));
        println!();
    }
    default_prepare!();
}

struct LoadValue {
    load: f32,
    warn: f32,
    bad: f32,
}

impl Display for LoadValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = if self.load >= self.bad {
            color::Fg(color::Red).to_string()
        } else if self.load >= self.warn {
            color::Fg(color::Yellow).to_string()
        } else {
            color::Fg(color::Green).to_string()
        };
        write!(f, "{color}")?;
        self.load.fmt(f)?;
        write!(f, "{}", style::Reset)?;
        Ok(())
    }
}

impl LoadAvg {
    pub fn print_or_error(self) -> Result<(), Box<dyn Error>> {
        let sys = System::new();
        let lavg = sys.load_average()?;
        let num_cpus = available_parallelism()?.get();
        let warn = self.warn_threshold.unwrap_or(num_cpus as f32);
        let bad = self.bad_threshold.unwrap_or((4 * num_cpus) as f32);

        // Vector which owns LoadValue objects
        let values: Vec<_> = [
            ("one", lavg.one),
            ("five", lavg.five),
            ("fifteen", lavg.fifteen),
        ]
        .into_iter()
        .map(|(k, v)| (k, LoadValue { load: v, warn, bad }))
        .collect();

        // interpolator::Context holding references to values
        let context: HashMap<_, _> = values
            .iter()
            .map(|(k, v)| (*k, Formattable::display(v)))
            .collect();

        println!("{}", interpolator::format(&self.format, &context)?);

        Ok(())
    }
}
