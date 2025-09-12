use serde_json::Value;
use std::{fs, io::{self, Write}, path::PathBuf};

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

/// Prompt user for API key if missing or invalid
pub async fn prompt_api_key(validate: impl Fn(&str) -> bool) -> String {
    loop {
        print!("Enter your YouTube API key: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let key = input.trim().to_string();

        if validate(&key) {
            save_api_key(&key);
            return key;
        } else {
            eprintln!("❌ API key not valid, try again.");
        }
    }
}

