use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    #[serde(default = "default_progress_character")]
    pub progress_full_character: char,
    #[serde(default = "default_progress_character")]
    pub progress_empty_character: char,
    #[serde(default = "default_progress_prefix")]
    pub progress_prefix: String,
    #[serde(default = "default_progress_suffix")]
    pub progress_suffix: String,
    #[serde(default = "default_progress_width")]
    pub progress_width: usize,
    #[serde(default = "default_time_format")]
    pub time_format: String,
}

fn default_progress_character() -> char {
    '='
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

// TODO: See if we can use this: https://github.com/serde-rs/serde/issues/1416
impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            progress_full_character: default_progress_character(),
            progress_empty_character: default_progress_character(),
            progress_prefix: default_progress_prefix(),
            progress_suffix: default_progress_suffix(),
            progress_width: default_progress_width(),
            time_format: default_time_format(),
        }
    }
}
