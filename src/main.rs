use std::env;

mod command;
mod components;
mod config;
mod constants;
use component::{Component, Constraints};
use config::get_config::get_config;
mod component;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args();

    match get_config(args) {
        Ok(config) => {
            let (components, constraints): (Vec<Box<dyn Component>>, Vec<Option<Constraints>>) =
                config
                    .components
                    .into_iter()
                    .map(|component| component.prepare(&config.global))
                    .unzip();

            let min_width = constraints
                .into_iter()
                .flatten()
                .filter_map(|x| x.min_width)
                .max();

            for component in components {
                component.print(&config.global, min_width).await;
            }
        }
        Err(e) => println!("Config Error: {}", e),
    }
    Ok(())
}
