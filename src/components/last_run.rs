use async_trait::async_trait;
use chrono::Local;
use serde::Deserialize;
use thiserror::Error;

use crate::{
    component::{Component, Constraints},
    config::global_config::GlobalConfig,
};

#[derive(Debug, Deserialize)]
pub struct LastRun {}

#[async_trait]
impl Component for LastRun {
    async fn print(self: Box<Self>, global_config: &GlobalConfig, _width: Option<usize>) {
        self.print_or_error(global_config)
            .unwrap_or_else(|err| println!("Last run error: {}", err));
    }
    fn prepare(
        self: Box<Self>,
        _global_config: &GlobalConfig,
    ) -> (Box<dyn Component>, Option<Constraints>) {
        (self, None)
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
