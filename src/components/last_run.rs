use chrono::Local;
use std::collections::HashMap;
use thiserror::Error;

// TODO: This is either enabled or disabled so figure out what to put here
// Emtpy struct doesn't work, but empty hashmap does
pub type LastRunConfig = HashMap<String, usize>;

#[derive(Error, Debug)]
pub enum LastRunError {
    #[error(transparent)]
    ChronoParseError(#[from] chrono::ParseError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub fn disp_last_run(_last_run_config: LastRunConfig) -> Result<(), LastRunError> {
    println!("Last updated: {}", Local::now().format("%Y/%m/%d %I:%M %P"));
    Ok(())
}
