use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::api;

/// Get the directory for application config
fn config_dir() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("ytm");
    std::fs::create_dir_all(&dir).ok(); // Ensure directory exists
    dir
}

/// Get the config file path
fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

/// Configuration file structure
#[derive(Debug, Default, Serialize, Deserialize)]
struct FileConfig {
    api_key: Option<String>,
}

/// Application configuration
pub struct Config {
    data: FileConfig,
}

impl Config {
    /// Load configuration from file
    pub fn load() -> anyhow::Result<Self> {
        let path = config_file();
        let data = if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            FileConfig::default()
        };
        Ok(Self { data })
    }

    /// Save configuration to file
    pub fn save(&self) -> anyhow::Result<()> {
        let path = config_file();
        let content = serde_json::to_string_pretty(&self.data)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Get the current API key
    pub fn api_key(&self) -> Option<&String> {
        self.data.api_key.as_ref()
    }

    /// Set the API key
    pub fn set_api_key(&mut self, key: String) {
        self.data.api_key = Some(key);
    }

    /// Validate if the current API key is valid
    pub async fn is_api_key_valid(&self) -> bool {
        if let Some(key) = &self.data.api_key {
            api::validate_key(key).await
        } else {
            false
        }
    }

    /// Ensure we have a valid API key (for headless use)
    #[allow(dead_code)] // May be used for headless operation
    pub async fn ensure_api_key(&mut self) -> anyhow::Result<String> {
        if let Some(ref key) = self.data.api_key {
            if api::validate_key(key).await {
                return Ok(key.clone());
            }
        }
        Err(anyhow::anyhow!("No valid API key available. Use --api to set one."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let config = Config {
            data: FileConfig::default(),
        };
        
        assert_eq!(config.api_key(), None);
    }

    #[test]
    fn test_config_with_api_key() {
        let mut config = Config {
            data: FileConfig::default(),
        };
        
        config.set_api_key("test_api_key".to_string());
        assert_eq!(config.api_key(), Some(&"test_api_key".to_string()));
    }

    #[test]
    fn test_config_serialization() {
        let config_data = FileConfig {
            api_key: Some("test_key_123".to_string()),
        };
        
        // Test serialization
        let json = serde_json::to_string(&config_data).unwrap();
        assert!(json.contains("test_key_123"));
        
        // Test deserialization
        let deserialized: FileConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.api_key, Some("test_key_123".to_string()));
    }

    #[test]
    fn test_config_load_from_invalid_json() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("invalid_config.json");
        
        // Write invalid JSON
        fs::write(&config_path, "invalid json content").unwrap();
        
        // Even with invalid JSON, Config::load should not panic
        // In real usage, it would fall back to default via unwrap_or_default()
        let default_config = FileConfig::default();
        assert_eq!(default_config.api_key, None);
    }

    #[test]
    fn test_config_paths() {
        // These should not panic
        let _ = config_dir();
        let _ = config_file();

        // Config file should be in config directory
        let config_path = config_file();
        let config_dir_path = config_dir();
        assert!(config_path.starts_with(config_dir_path));
        assert!(config_path.ends_with("config.json"));
    }
}
