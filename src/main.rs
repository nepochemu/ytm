use clap::{Parser, Subcommand};
use reqwest;
use serde_json::Value;
use std::{error::Error, fs, process::Command};

#[derive(Parser)]
#[command(name = "youtube-mpc", about = "Search and play YouTube via mpv")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search YouTube for videos
    Search {
        /// Query words (no quotes needed)
        #[arg(num_args(1..), trailing_var_arg = true)]
        query: Vec<String>,
    },
    /// Play a video by its index from the last search
    Play {
        /// 1-based index from the last search results
        index: usize,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search { query } => {
            // 🔑 Inline your API key here
            let api_key = "AIzaSyD1wRjl2XpxxU6g1ts8rSGRXegs8g-A50s";

            let q = query.join(" ");
            let url = format!(
                "https://www.googleapis.com/youtube/v3/search?part=snippet&type=video&maxResults=5&q={}&key={}",
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
            Command::new("mpv")
                .arg(&url)
                .status()
                .expect("failed to launch mpv");
        }
    }

    Ok(())
}

