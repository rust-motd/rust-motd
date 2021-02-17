use humantime::format_duration;
use serde::Deserialize;
use systemstat::{Platform, System};

#[derive(Debug, Deserialize)]
pub struct UptimeCfg {
    prefix: String,
}

pub fn disp_uptime(config: UptimeCfg, sys: &System) -> Result<(), std::io::Error> {
    let uptime = sys.uptime()?;
    println!("{} {}", config.prefix, format_duration(uptime).to_string());
    Ok(())
}
