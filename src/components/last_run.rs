use async_trait::async_trait;
use chrono::Local;
use serde::Deserialize;
use thiserror::Error;

use crate::{component::Component, config::global_config::GlobalConfig};

#[derive(Debug, Deserialize)]
pub struct LastRun {}

#[async_trait]
impl Component for LastRun {
    async fn print(self: Box<Self>, global_config: &GlobalConfig) {
        self.print_or_error(global_config)
            .unwrap_or_else(|err| println!("Last run error: {}", err));
    }
}

#[derive(Error, Debug)]
pub enum LastRunError {
    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

impl LastRun {
    pub fn print_or_error(self, global_config: &GlobalConfig) -> Result<(), LastRunError> {
        println!(
            "Last updated: {}",
            Local::now().format(&global_config.time_format)
        );
        Ok(())
    }
}
