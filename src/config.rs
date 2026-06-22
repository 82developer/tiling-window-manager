use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct LayoutConfig {
    #[serde(default = "default_gap")]
    pub gap: i32,
    #[serde(default = "default_margin")]
    pub margin: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TerminalConfig {
    #[serde(default = "default_terminal_command")]
    pub command: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IgnoreConfig {
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub titles: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RuleEntry {
    #[serde(default)]
    pub class: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub workspace: usize,
    #[serde(default)]
    pub floating: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub layout: LayoutConfig,
    #[serde(default)]
    pub terminal: TerminalConfig,
    #[serde(default)]
    pub ignore: IgnoreConfig,
    #[serde(default)]
    pub hotkeys: HashMap<String, String>,
    #[serde(default)]
    pub rules: Vec<RuleEntry>,
}

fn default_gap() -> i32 { 8 }
fn default_margin() -> i32 { 0 }
fn default_terminal_command() -> String { "wt.exe".to_string() }

impl Default for LayoutConfig {
    fn default() -> Self {
        Self { gap: default_gap(), margin: default_margin() }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self { command: default_terminal_command() }
    }
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            classes: vec![
                "Shell_TrayWnd".to_string(),
                "Progman".to_string(),
                "Windows.UI.Core.CoreWindow".to_string(),
                "ApplicationFrameWindow".to_string(),
            ],
            titles: vec!["Program Manager".to_string(), "".to_string()],
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layout: LayoutConfig::default(),
            terminal: TerminalConfig::default(),
            ignore: IgnoreConfig::default(),
            hotkeys: HashMap::new(),
            rules: Vec::new(),
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> crate::error::AppResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::error::AppError::Config(format!("failed to read config file '{}': {}", path, e))
        })?;
        let config: Config = toml::from_str(&content).map_err(|e| {
            crate::error::AppError::Config(format!("failed to parse config file: {}", e))
        })?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_rules() {
        let toml_str = r#"
[layout]
gap = 4

[hotkeys]
quit = "CTRL+ALT+ESC"

[[rules]]
class = "CASCADIA*"
workspace = 1

[[rules]]
title = "Notepad"
workspace = 2
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.layout.gap, 4);
        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules[0].class.as_ref().unwrap(), "CASCADIA*");
        assert_eq!(config.rules[0].workspace, 1);
        assert_eq!(config.rules[1].title.as_ref().unwrap(), "Notepad");
    }

    #[test]
    fn test_parse_default_config() {
        let toml_str = r#"
[hotkeys]
move_left = "CTRL+ALT+H"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.layout.gap, 8);
        assert!(config.rules.is_empty());
    }
}
