use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, process::Command};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Search { query: String },
    Play { video_id: String },
    Select { index: usize },
}

#[derive(Debug, Serialize, Deserialize)]
struct YoutubeSearchResponse {
    items: Vec<YoutubeItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YoutubeItem {
    id: YoutubeId,
    snippet: Snippet,
}

#[derive(Debug, Serialize, Deserialize)]
struct YoutubeId {
    #[serde(rename = "videoId")]
    video_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Snippet {
    title: String,
    #[serde(rename = "channelTitle")]
    channel_title: String,
}

fn cache_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or(".".to_string());
    PathBuf::from(home).join(".youtube-mpc-last.json")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ⚠️ replace this with your real API key
    let api_key = "AIzaSyD1wRjl2XpxxU6g1ts8rSGRXegs8g-A50s";

    let cli = Cli::parse();

    match cli.command {
        Commands::Search { query } => {
            let url = format!(
                "https://www.googleapis.com/youtube/v3/search?part=snippet&type=video&maxResults=5&q={}&key={}",
                query, api_key
            );

            let resp = reqwest::get(&url).await?;
            let text = resp.text().await?;

            match serde_json::from_str::<YoutubeSearchResponse>(&text) {
                Ok(results) => {
                    // print results
                    for (i, item) in results.items.iter().enumerate() {
                        println!(
                            "{}: {} [{}] (watch?v={})",
                            i + 1,
                            item.snippet.title,
                            item.snippet.channel_title,
                            item.id.video_id
                        );
                    }

                    // cache results
                    let cache = cache_path();
                    fs::write(cache, serde_json::to_string(&results)?)?;
                }
                Err(e) => {
                    eprintln!("❌ Failed to parse YouTube API JSON: {}", e);
                    eprintln!("Raw response:\n{}", text);
                }
            }
        }

        Commands::Play { video_id } => {
            let url = format!("https://youtube.com/watch?v={}", video_id);
            println!("Playing {}", url);

            Command::new("mpv")
                .arg("--no-video")
                .arg(&url)
                .status()?;
        }

        Commands::Select { index } => {
            let cache = cache_path();
            if let Ok(data) = fs::read_to_string(cache) {
                if let Ok(results) = serde_json::from_str::<YoutubeSearchResponse>(&data) {
                    if index > 0 && index <= results.items.len() {
                        let video = &results.items[index - 1];
                        let url = format!("https://youtube.com/watch?v={}", video.id.video_id);
                        println!("Playing: {} [{}]", video.snippet.title, video.snippet.channel_title);

                        Command::new("mpv")
                            .arg("--no-video")
                            .arg(&url)
                            .status()?;
                    } else {
                        eprintln!("❌ Invalid index: {}", index);
                    }
                }
            } else {
                eprintln!("❌ No cached search results found. Run `search` first.");
            }
        }
    }

    Ok(())
}

