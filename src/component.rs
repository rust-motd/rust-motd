use crate::config::global_config::GlobalConfig;
use async_trait::async_trait;

pub type BoxedComponent = Box<dyn Component + Send>;
pub type PrepareReturn = (BoxedComponent, Option<Constraints>);

pub struct Constraints {
    pub min_width: Option<usize>,
}

#[async_trait]
pub trait Component {
    // Prepare the component and return its sizing constraints
    // This also returns another Component, which allows a new struct implementing this trait to be
    // returned with all of the prepared data
    // This saves from doing expensive preparation twice and is better and easier than mutating
    // self
    // For example, check `PreparedFilesystems`
    // Otherwise, simply return `self` if there is no data to save from the preparation phase
    fn prepare(self: Box<Self>, _global_config: &GlobalConfig) -> PrepareReturn;

    // Print the component to stdout
    async fn print(self: Box<Self>, global_config: &GlobalConfig, width: Option<usize>);
}
