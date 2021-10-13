use chrono::Local;
use thiserror::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LastRunConfig {}

#[derive(Error, Debug)]
pub enum LastRunError {
    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

pub fn disp_last_run(_last_run_config: LastRunConfig) -> Result<(), LastRunError> {
    println!("Last updated: {}", Local::now().format("%Y/%m/%d %I:%M %P"));
    Ok(())
}
