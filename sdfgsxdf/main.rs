mod api;
mod config;
mod commands;
mod cache; // ✅ add cache module

use clap::{Parser, ArgAction};
use std::error::Error;
use crate::cache::Cache;

#[derive(Parser)]
#[command(
    name = "ytm",
    version,
    about = "Search and play YouTube via mpv + fzf"
)]
struct Cli {
    /// Print version (-v or --version)
    #[arg(short = 'v', long = "version", action = ArgAction::Version)]
    version: Option<bool>,

    /// Audio only (no video)
    #[arg(short = 'n', long)]
    audio_only: bool,

    /// Search terms (e.g. `ytm chlär boiler room`)
    query: Vec<String>,

    /// Set or replace API key
    #[arg(long)]
    api: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // ✅ Initialize cache once at startup
    let cache = Cache::new(
        dirs::cache_dir().unwrap().join("ytm"),
        std::time::Duration::from_secs(60 * 60 * 24), // 1 day TTL
    )?;

    // ✅ Build the YouTube client with API key + cache
    let client = api::YouTubeClient::new(config::get_api_key()?, cache);

    // ✅ Handle API key setup
    if let Some(key) = cli.api {
        commands::set_api_key(key).await?;
        return Ok(());
    }

    // ✅ Handle missing query
    if cli.query.is_empty() {
        eprintln!("❌ No search query given.");
        eprintln!("Usage: ytm [-n] <search terms>  or  ytm --api <key>");
        return Ok(());
    }

    // ✅ Run the search, passing in the client
    commands::search(&client, cli.query, cli.audio_only).await
}
