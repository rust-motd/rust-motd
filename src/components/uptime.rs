use humantime::format_duration;
use serde::Deserialize;
use systemstat::{Platform, System};

#[derive(Debug, Deserialize)]
pub struct UptimeConfig {
    prefix: String,
}

pub fn disp_uptime(config: UptimeConfig, sys: &System) -> Result<(), std::io::Error> {
    let uptime = sys.uptime()?;
    println!("{} {}", config.prefix, format_duration(uptime));

    Ok(())
}
