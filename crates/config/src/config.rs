mod default;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    pub general: General,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: General::default(),
        }
    }
}

/// Todo: Modify orig
#[derive(Deserialize)]
struct General {
    pub ssh_config_path: String,
}

impl Default for General {
    fn default() -> Self {
        Self {
            ssh_config_path: default::ssh_config_path(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.general.ssh_config_path.is_empty());
    }
}
