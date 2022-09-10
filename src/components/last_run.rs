use chrono::Local;
use serde::Deserialize;
use thiserror::Error;

use crate::config::global_config::GlobalConfig;

#[derive(Debug, Deserialize)]
pub struct LastRunConfig {}

#[derive(Error, Debug)]
pub enum LastRunError {
    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

pub fn disp_last_run(
    _last_run_config: LastRunConfig,
    global_config: &GlobalConfig,
) -> Result<(), LastRunError> {
    println!(
        "Last updated: {}",
        Local::now().format(&global_config.time_format)
    );
    Ok(())
}
