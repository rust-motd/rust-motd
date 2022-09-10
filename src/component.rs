use crate::config::global_config::GlobalConfig;
use async_trait::async_trait;

#[async_trait]
pub trait Component {
    async fn print(self: Box<Self>, global_config: &GlobalConfig);
}
