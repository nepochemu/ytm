use clap::{Parser, Subcommand};
use reqwest;
use serde_json::Value;
use std::{
    error::Error,
    fs,
    io::{self, Write},
    path::PathBuf,
    process::Command,
};

/// Location of config file (~/.config/ytm/config.json)
fn config_path() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("ytm");
    fs::create_dir_all(&dir).ok();
    dir.push("config.json");
    dir
}

/// Load API key from config, or prompt if missing
async fn get_api_key() -> String {
    let path = config_path();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<Value>(&data) {
                if let Some(key) = v.get("api_key").and_then(|v| v.as_str()) {
                    // validate key before returning
                    if validate_key(key).await {
                        return key.to_string();
                    } else {
                        eprintln!("⚠️ Stored API key is invalid.");
                    }
                }
            }
        }
    }
    // If no valid key, prompt user
    loop {
        print!("Enter your YouTube API key: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let key = input.trim().to_string();

        if validate_key(&key).await {
            save_api_key(&key);
            return key;
        } else {
            eprintln!("❌ API key not valid, try again.");
        }
    }
}

/// Save API key to config file
fn save_api_key(key: &str) {
    let path = config_path();
    let json = serde_json::json!({ "api_key": key });
    fs::write(path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
    println!("✅ API key saved.");
}

/// Quick check if API key works
async fn validate_key(key: &str) -> bool {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&type=video&maxResults=1&q=test&key={}",
        key
    );
    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => true,
        _ => false,
    }
}

/// Simple heuristic for when audio-only makes sense even without the flag
fn should_force_no_video(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    // obvious audio file extensions
    let audio_ext = [".mp3", ".flac", ".wav", ".aac", ".m4a", ".ogg", ".opus"];
    if audio_ext.iter().any(|ext| lower.ends_with(ext)) {
        return true;
    }
    // common music-only sources
    lower.contains("music.youtube.com") || lower.contains("soundcloud.com") || lower.contains("bandcamp.com")
}

#[derive(Parser)]
#[command(name = "ytm", about = "Search and play YouTube via mpv")]
struct Cli {
    /// Play audio only (no video window). Short: -n
    #[arg(short = 'n', long = "no-video", global = true)]
    no_video: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search YouTube for videos
    Search {
        #[arg(num_args(1..), trailing_var_arg = true)]
        query: Vec<String>,
    },
    /// Play a video by its index from the last search
    Play {
        index: usize,
    },
    /// Set or replace API key
    Api {
        key: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search { query } => {
            let api_key = get_api_key().await;

            let q = query.join(" ");
            let url = format!(
                "https://www.googleapis.com/youtube/v3/search?part=snippet&type=video&maxResults=20&q={}&key={}",
                q, api_key
            );

            let resp_text = reqwest::get(&url).await?.text().await?;
            let resp: Value = serde_json::from_str(&resp_text)?;

            let items: Vec<Value> = resp
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            for (i, item) in items.iter().enumerate() {
                let video_id = item["id"]["videoId"].as_str().unwrap_or("");
                let title = item["snippet"]["title"].as_str().unwrap_or("");
                let channel = item["snippet"]["channelTitle"].as_str().unwrap_or("");
                println!("{}: {} [{}] (watch?v={})", i + 1, title, channel, video_id);
            }

            fs::write("last_results.json", serde_json::to_string(&resp)?)?;
        }

        Commands::Play { index } => {
            let data = fs::read_to_string("last_results.json")
                .expect("No cached results. Run `search` first.");
            let resp: Value = serde_json::from_str(&data)?;

            let items: Vec<Value> = resp
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            if index == 0 || index > items.len() {
                eprintln!("Invalid index. Use 1..{}", items.len());
                return Ok(());
            }

            let video_id = items[index - 1]["id"]["videoId"]
                .as_str()
                .unwrap_or("");
            if video_id.is_empty() {
                eprintln!("Selected item has no videoId.");
                return Ok(());
            }

            let url = format!("https://youtube.com/watch?v={}", video_id);
            println!("Playing {}", url);

            // Build mpv command with optional --no-video
            let mut cmd = Command::new("mpv");

            if cli.no_video || should_force_no_video(&url) {
                cmd.arg("--no-video");
            }

            cmd.arg(&url)
                .status()
                .expect("failed to launch mpv");
        }

        Commands::Api { key } => {
            if validate_key(&key).await {
                save_api_key(&key);
            } else {
                eprintln!("❌ Provided API key is not valid.");
            }
        }
    }

    Ok(())
}

