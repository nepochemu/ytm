mod api;
mod config;
mod commands;

use clap::Parser;
use std::error::Error;

#[derive(Parser)]
#[command(name = "ytm", about = "Search and play YouTube via mpv + fzf")]
struct Cli {
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

    if let Some(key) = cli.api {
        commands::set_api_key(key).await?;
        return Ok(());
    }

    if cli.query.is_empty() {
        eprintln!("❌ No search query given.");
        eprintln!("Usage: ytm [-n] <search terms>  or  ytm --api <key>");
        return Ok(());
    }

    commands::search(cli.query, cli.audio_only).await
}

