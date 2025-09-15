use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};
use serde_json::Value;

use crate::api; // for validate_key

/// Location of config file (~/.config/ytm/config.json)
pub fn config_path() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("ytm");
    fs::create_dir_all(&dir).ok();
    dir.push("config.json");
    dir
}

/// Save API key to config file
pub fn save_api_key(key: &str) {
    let path = config_path();
    let json = serde_json::json!({ "api_key": key });
    fs::write(path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
    println!("✅ API key saved.");
}

/// Prompt user for API key until valid
pub async fn prompt_api_key() -> String {
    loop {
        print!("Enter your YouTube API key: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let key = input.trim().to_string();

        if api::validate_key(&key).await {
            save_api_key(&key);
            return key;
        } else {
            eprintln!("[!] API key not valid, try again.");
        }
    }
}

/// Load saved API key if present and valid, otherwise prompt
pub async fn load_or_prompt_api_key() -> String {
    let path = config_path();

    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(json) = serde_json::from_str::<Value>(&data) {
            if let Some(key) = json["api_key"].as_str() {
                if api::validate_key(key).await {
                    return key.to_string();
                }
            }
        }
    }

    // fallback: ask user
    prompt_api_key().await
}

