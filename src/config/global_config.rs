use knuffel;
use serde::Deserialize;

#[derive(Debug, Deserialize, knuffel::Decode)]
pub struct GlobalConfig {
    #[knuffel(child, unwrap(argument))]
    pub version: Option<String>,

    #[knuffel(child, unwrap(argument), default=default_progress_character())]
    #[serde(default = "default_progress_character")]
    pub progress_full_character: String,

    #[knuffel(child, unwrap(argument), default=default_progress_character())]
    #[serde(default = "default_progress_character")]
    pub progress_empty_character: String,

    #[knuffel(child, unwrap(argument), default=default_progress_prefix())]
    #[serde(default = "default_progress_prefix")]
    pub progress_prefix: String,

    #[knuffel(child, unwrap(argument), default=default_progress_suffix())]
    #[serde(default = "default_progress_suffix")]
    pub progress_suffix: String,

    #[knuffel(child, unwrap(argument), default=default_progress_width())]
    #[serde(default = "default_progress_width")]
    pub progress_width: usize,

    #[knuffel(child, unwrap(argument), default=default_time_format())]
    #[serde(default = "default_time_format")]
    pub time_format: String,

    #[serde(default = "default_show_legacy_warning")]
    pub show_legacy_warning: bool,
}

fn default_progress_character() -> String {
    "=".to_string()
}

fn default_progress_prefix() -> String {
    "[".to_string()
}

fn default_progress_suffix() -> String {
    "]".to_string()
}

fn default_progress_width() -> usize {
    80
}

fn default_time_format() -> String {
    "%Y-%m-%d %H:%M:%S %Z".to_string()
}

fn default_show_legacy_warning() -> bool {
    true
}

// TODO: See if we can use this: https://github.com/serde-rs/serde/issues/1416
impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            version: None,
            progress_full_character: default_progress_character(),
            progress_empty_character: default_progress_character(),
            progress_prefix: default_progress_prefix(),
            progress_suffix: default_progress_suffix(),
            progress_width: default_progress_width(),
            time_format: default_time_format(),
            show_legacy_warning: default_show_legacy_warning(),
        }
    }
}
