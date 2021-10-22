use serde::Deserialize;

pub const INDENT_WIDTH: usize = 2;

#[derive(Debug, Deserialize)]
pub struct GlobalSettings {
    #[serde(default = "default_progress_character")]
    pub progress_full_character: char,
    #[serde(default = "default_progress_character")]
    pub progress_empty_character: char,
    #[serde(default = "default_progress_prefix")]
    pub progress_prefix: String,
    #[serde(default = "default_progress_suffix")]
    pub progress_suffix: String,
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

// TODO: See if we can use this: https://github.com/serde-rs/serde/issues/1416
impl Default for GlobalSettings {
    fn default() -> Self {
        GlobalSettings {
            progress_full_character: default_progress_character(),
            progress_empty_character: default_progress_character(),
            progress_prefix: default_progress_prefix(),
            progress_suffix: default_progress_suffix(),
        }
    }
}
