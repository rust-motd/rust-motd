use std::env;

mod command;
mod components;
mod config;
mod constants;
use config::get_config::get_config;
mod component;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args();

    match get_config(args) {
        Ok(config) => {
            for component in config.components {
                component.print(&config.global).await;
            }
        }
        Err(e) => println!("Config Error: {}", e),
    }
    Ok(())
}
