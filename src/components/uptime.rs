use humantime::format_duration;

pub mod components::uptime;

#[derive(Debug, Deserialize)]
struct UptimeCfg {
    prefix: String,
}

fn disp_uptime(config: UptimeCfg, sys: &System) -> Result<(), std::io::Error> {
    let uptime = sys.uptime()?;
    println!("{} {}", config.prefix, format_duration(uptime).to_string());
    Ok(())
}
