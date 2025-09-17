mod api;
mod cache;
mod commands;
mod config;

use crate::cache::Cache;
use clap::{ArgAction, Parser};
use std::error::Error;

#[derive(Parser)]
#[command(name = "ytm", version, about = "Search and play YouTube via mpv + fzf")]
struct Cli {
    #[arg(short = 'v', long = "version", action = ArgAction::Version)]
    version: Option<bool>,

    #[arg(short = 'n', long)]
    audio_only: bool,

    query: Vec<String>,

    #[arg(long)]
    api: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let cache = Cache::new(
        dirs::cache_dir().unwrap().join("ytm"),
        std::time::Duration::from_secs(60 * 60 * 5), // 1 day
    )?;

    let api_key = config::load_or_prompt_api_key().await;
    let client = api::YouTubeClient::new(api_key, cache);

    if let Some(key) = cli.api {
        commands::set_api_key(key).await?;
        return Ok(());
    }

    if cli.query.is_empty() {
        eprintln!("[!] No search query given.");
        eprintln!("Usage: ytm [-n] <search terms>  or  ytm --api <key>");
        return Ok(());
    }

    commands::search(&client, cli.query, cli.audio_only).await
}
