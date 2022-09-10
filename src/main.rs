use std::env;

mod command;
mod components;
mod config;
mod constants;
use component::{BoxedComponent, Constraints};
use config::get_config::get_config;
mod component;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args();

    match get_config(args) {
        Ok(config) => {

            // Run the prepare phase for each component
            // Allow each component to specify its sizing constraints (like min width)
            let (components, constraints): (Vec<BoxedComponent>, Vec<Option<Constraints>>) = config
                .components
                .into_iter()
                .map(|component| component.prepare(&config.global))
                .unzip();

            // The width to use is the maximum of all the component's minimum widths
            // Right now, min width is the only constraint
            let width = constraints
                .into_iter()
                .flatten()
                .filter_map(|x| x.min_width)
                .max();

            // Print each component with the given width
            for component in components {
                component.print(&config.global, width).await;
            }
        }
        Err(e) => println!("Config Error: {}", e),
    }
    Ok(())
}
