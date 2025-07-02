pub mod get_config;
pub mod global_config;
pub mod kdl_config;
pub mod toml_config;

use crate::component::BoxedComponent;
use global_config::GlobalConfig;

/// Configuration for all components and the global settings
/// The order of the components in the vector is the order they appear in the configuration file
/// and is the order in which they should be printed
/// This way, users can configure the order of components by shifting lines in the config file
pub struct Config {
    pub components: Vec<BoxedComponent>,
    pub global: GlobalConfig,
}
