use async_trait::async_trait;
use humantime::format_duration;
use serde::Deserialize;
use systemstat::{Platform, System};

use crate::component::{Component, PrepareReturn};
use crate::config::global_config::GlobalConfig;

#[derive(Debug, Deserialize)]
pub struct Uptime {
    prefix: String,
}

#[async_trait]
impl Component for Uptime {
    async fn print(self: Box<Self>, _global_config: &GlobalConfig, _width: Option<usize>) {
        self.print_or_error()
            .unwrap_or_else(|err| println!("Uptime error: {}", err));
        println!();
    }
    fn prepare(self: Box<Self>, _global_config: &GlobalConfig) -> PrepareReturn {
        (self, None)
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
