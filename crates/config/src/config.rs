mod default;
use serde::Deserialize;
use std::{fmt::Debug, path::PathBuf};

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct AppConfig {
    pub general: Option<General>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: Some(General::default()),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct General {
    pub ssh_config_path: Option<String>,
}

impl Default for General {
    fn default() -> Self {
        Self {
            ssh_config_path: Some(default::ssh_config_path()),
        }
    }
}

pub fn read_config(path: &PathBuf) -> AppConfig {
    let contents = std::fs::read_to_string(&path).expect("Failed to read config file");
    let config: AppConfig = toml::from_str(&contents).expect("Failed to parse TOML");
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(config.general.is_some());
        assert!(config.general.as_ref().unwrap().ssh_config_path.is_some());
    }

    #[test]
    fn test_partial_config() {
        // Test parsing of incomplete TOML
        let partial_config = r#"
            [general]
        "#;
        let config: AppConfig = toml::from_str(partial_config).unwrap();
        assert!(config.general.is_some());
        assert!(config.general.as_ref().unwrap().ssh_config_path.is_some());
    }
}
