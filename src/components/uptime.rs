use async_trait::async_trait;
use humantime::format_duration;
use serde::Deserialize;
use systemstat::{Platform, System};

use crate::component::Component;
use crate::config::global_config::GlobalConfig;

#[derive(Debug, Deserialize)]
pub struct Uptime {
    prefix: String,
}

#[async_trait]
impl Component for Uptime {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig) {
        self.print_or_error()
            .unwrap_or_else(|err| println!("Uptime error: {}", err));
        println!();
    }
}

impl Uptime {
    pub fn print_or_error(self) -> Result<(), std::io::Error> {
        let sys = System::new();
        let uptime = sys.uptime()?;
        println!("{} {}", self.prefix, format_duration(uptime));

        Ok(())
    }
}
