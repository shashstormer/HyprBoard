use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::view::components::theme::AppTheme;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub plugins: Vec<String>,
    #[serde(default)]
    pub theme: AppTheme,
}

impl Config {
    pub fn load() -> Self {
        let config_path = Path::new("config.json");
        if config_path.exists() {
            let content = fs::read_to_string(config_path).unwrap_or_else(|_| "{}".to_string());
            serde_json::from_str(&content).unwrap_or_else(|_| Config::default())
        } else {
            Config::default()
        }
    }

    pub fn save(&self) {
        let config_path = Path::new("config.json");
        let content = serde_json::to_string_pretty(self).unwrap_or_default();
        let _ = fs::write(config_path, content);
    }

    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            theme: AppTheme::default(),
        }
    }
}
