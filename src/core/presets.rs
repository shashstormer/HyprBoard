use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub description: String,
}

pub struct PresetManager {
    category: String,
    base_dir: PathBuf,
}

impl PresetManager {
    pub fn new(category: &str) -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let path = PathBuf::from(home)
            .join(".config/hyprboard/presets")
            .join(category);
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        Self {
            category: category.to_string(),
            base_dir: path,
        }
    }

    pub fn list(&self) -> Vec<Preset> {
        let mut presets = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(stem) = path.file_name().and_then(|s| s.to_str()) {
                        if !stem.starts_with('.') {
                            presets.push(Preset {
                                name: stem.to_string(),
                                description: format!("Preset {}", stem),
                            });
                        }
                    }
                }
            }
        }
        presets.sort_by(|a, b| a.name.cmp(&b.name));
        presets
    }

    pub fn save(&self, name: &str, files: &HashMap<String, String>) -> Result<(), String> {
        let preset_dir = self.base_dir.join(name);
        if !preset_dir.exists() {
            fs::create_dir_all(&preset_dir).map_err(|e| e.to_string())?;
        }

        for (filename, content) in files {
            fs::write(preset_dir.join(filename), content).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn load(&self, name: &str) -> Result<HashMap<String, String>, String> {
        let preset_dir = self.base_dir.join(name);
        if !preset_dir.exists() {
            return Err("Preset not found".to_string());
        }

        let mut files = HashMap::new();
        if let Ok(entries) = fs::read_dir(&preset_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            files.insert(filename.to_string(), content);
                        }
                    }
                }
            }
        }
        Ok(files)
    }

    pub fn delete(&self, name: &str) -> Result<(), String> {
        let preset_dir = self.base_dir.join(name);
        if preset_dir.exists() {
            fs::remove_dir_all(preset_dir).map_err(|e| e.to_string())
        } else {
            Err("Preset not found".to_string())
        }
    }

    pub fn get_active(&self) -> Option<String> {
        let path = self.base_dir.join(".active");
        if path.exists() {
            fs::read_to_string(path)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        } else {
            None
        }
    }

    pub fn set_active(&self, name: Option<&str>) -> Result<(), String> {
        let path = self.base_dir.join(".active");
        if let Some(n) = name {
            fs::write(path, n).map_err(|e| e.to_string())
        } else {
            if path.exists() {
                let _ = fs::remove_file(path);
            }
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub name: String,
    pub items: HashMap<String, String>,
}

pub struct BundleManager {
    base_dir: PathBuf,
}

impl BundleManager {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let path = PathBuf::from(home).join(".config/hyprboard/bundles");
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        Self { base_dir: path }
    }

    pub fn list(&self) -> Vec<Bundle> {
        let mut bundles = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(bundle) = serde_json::from_str::<Bundle>(&content) {
                            bundles.push(bundle);
                        }
                    }
                }
            }
        }
        bundles.sort_by(|a, b| a.name.cmp(&b.name));
        bundles
    }

    pub fn save(&self, bundle: &Bundle) -> Result<(), String> {
        let path = self.base_dir.join(format!("{}.json", bundle.name));
        let content = serde_json::to_string_pretty(bundle).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn delete(&self, name: &str) -> Result<(), String> {
        let path = self.base_dir.join(format!("{}.json", name));
        if path.exists() {
            fs::remove_file(path).map_err(|e| e.to_string())
        } else {
            Err("Bundle not found".to_string())
        }
    }
}
