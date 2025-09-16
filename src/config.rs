use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};
use serde_json::Value;

use crate::api;

/// Location of config file (~/.config/ytm/config.json)
fn config_path() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("ytm");
    fs::create_dir_all(&dir).ok();
    dir.push("config.json");
    dir
}

pub struct Config {
    pub api_key: Option<String>,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = config_path();
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(json) = serde_json::from_str::<Value>(&data) {
                return Ok(Self {
                    api_key: json["api_key"].as_str().map(|s| s.to_string()),
                });
            }
        }
        Ok(Self { api_key: None })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = config_path();
        let json = serde_json::json!({ "api_key": self.api_key });
        fs::write(path, serde_json::to_string_pretty(&json)?)?;
        Ok(())
    }

    pub async fn ensure_api_key(&mut self) -> anyhow::Result<String> {
        if let Some(ref key) = self.api_key {
            if api::validate_key(key).await {
                return Ok(key.clone());
            }
        }
        loop {
            print!("Enter your YouTube API key: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let key = input.trim().to_string();
            if api::validate_key(&key).await {
                self.api_key = Some(key.clone());
                self.save()?;
                return Ok(key);
            } else {
                eprintln!("[!] Invalid key, try again.");
            }
        }
    }
}
