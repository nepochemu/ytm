use clap::{Parser, Subcommand};

mod api;
mod cache;
mod commands;
mod config;
mod mpv;

#[derive(Parser)]
#[command(name = "ytm")]
#[command(about = "YouTube terminal music player")]
#[command(version)]
#[command(after_help = "While in -b (background) mode, use ytm pause/resume/next/prev/stop/status commands to control background playback")]
struct Cli {
    /// Search term (shortcut for `ytm search <term>`) - supports multiple words
    query: Vec<String>,

    /// Enable video window (default is audio-only)
    #[arg(short = 'v', long, help = "Enable video window (default is audio-only)")]
    video: bool,

    /// Run player in background and return to terminal (enables pause/resume/next/prev/stop/status commands)
    #[arg(short = 'b', long, help = "Run player in background (enables pause/resume/next/prev/stop/status commands)")]
    background: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(hide = true)]
    Search {
        query: String,
        #[arg(short, long)]
        api: Option<String>,
    },
    #[command(hide = true)]
    Play { url: String },
    #[command(hide = true)]
    Pause,
    #[command(hide = true)]
    Next,
    #[command(hide = true)]
    Prev,
    #[command(hide = true)]
    Stop,
    #[command(hide = true)]
    Status,
}

/// Check if a query string is a player control command
fn is_control_command(query: &str) -> bool {
    matches!(query, "stop" | "pause" | "resume" | "next" | "prev" | "status")
}

/// Execute a control command on the running player
fn execute_control_command(cmd: &str) -> anyhow::Result<()> {
    match cmd {
        "stop" => commands::stop(),
        "pause" => commands::pause(),
        "resume" => commands::resume(),
        "next" => commands::next(),
        "prev" => commands::prev(),
        "status" => commands::status(),
        _ => Err(anyhow::anyhow!("Unknown control command: {}", cmd)),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // By default, audio-only. -v enables video.
    let no_video = !cli.video;

    if !cli.query.is_empty() {
        let query = cli.query.join(" ");
        
        // Check if this is a control command and mpv is running
        if is_control_command(&query) {
            if mpv::is_running() {
                execute_control_command(&query)?;
                std::process::exit(0); // Exit immediately after control command
            } else {
                eprintln!("Debug: mpv not detected as running, searching YouTube for '{}'", query);
            }
        }
        return commands::search_and_play(&query, None, no_video, cli.background).await;
    }

    match cli.command {
        Some(Commands::Search { query, api }) => {
            commands::search_and_play(&query, api, no_video, cli.background).await
        }
        Some(Commands::Play { url }) => {
            commands::play(&url, no_video, cli.background)
        }
        Some(Commands::Pause) => commands::pause(),
        Some(Commands::Next) => commands::next(),
        Some(Commands::Prev) => commands::prev(),
        Some(Commands::Stop) => commands::stop(),
        Some(Commands::Status) => commands::status(),
        None => {
            eprintln!("Usage: ytm <query> or ytm search <query>");
            Ok(())
        }
    }
}
