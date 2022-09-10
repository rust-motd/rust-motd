use crate::config::global_config::GlobalConfig;
use async_trait::async_trait;

/// Boxed component with all other traits
// Send is required because print is async
pub type BoxedComponent = Box<dyn Component + Send>;

/// Return type for the prepare phase
pub type PrepareReturn = (BoxedComponent, Option<Constraints>);

/// Constraints are optionally returned by the prepare phase
/// and allow a component to specify its sizing constraints
/// For example, the `Filesystems` component has a minimum width based on the length in characters
/// of its mount points and other data, and this width is used by the memory component
/// (aligning the progress bars like this is aesthetically pleasing)
pub struct Constraints {
    pub min_width: Option<usize>,
}

/// This trait should be implemented for all components
/// (component being all the things the motd can print like banner, memory, etc.).
#[async_trait]
pub trait Component {
    /// Prepare the component and return its sizing constraints
    /// This also returns another Component, which allows a new struct implementing this trait to be
    /// returned with all of the prepared data
    /// This saves from doing expensive preparation twice and is better and easier than mutating
    /// self
    /// For example, check `PreparedFilesystems`
    /// Otherwise, simply return `self` if there is no data to save from the preparation phase
    fn prepare(self: Box<Self>, _global_config: &GlobalConfig) -> PrepareReturn;

    /// Print the component to stdout
    async fn print(self: Box<Self>, global_config: &GlobalConfig, width: Option<usize>);
}
