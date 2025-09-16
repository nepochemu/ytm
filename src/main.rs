use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Search and play query
    pub query: Option<String>,

    /// API key override
    #[arg(long)]
    pub api: Option<String>,

    /// Background mode (release terminal, control with pause/stop/next/prev)
    #[arg(short, long)]
    pub background: bool,

    #[command(subcommand)]
    pub cmd: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Pause,
    Stop,
    Next,
    Prev,
    Status,
}

fn try_clipboard_query() -> Option<String> {
    use copypasta::{ClipboardContext, ClipboardProvider};
    let mut ctx = ClipboardContext::new().ok()?;
    let text = ctx.get_contents().ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Some(Commands::Pause)  => commands::pause()?,
        Some(Commands::Stop)   => commands::stop()?,
        Some(Commands::Next)   => commands::next()?,
        Some(Commands::Prev)   => commands::prev()?,
        Some(Commands::Status) => commands::status()?,
        None => {
            if let Some(query) = cli.query {
                commands::search_and_play(&query, cli.api, cli.background)?;
            } else if let Some(q) = try_clipboard_query() {
                commands::search_and_play(&q, cli.api, cli.background)?;
            } else {
                commands::status()?;
            }
        }
    }

    Ok(())
}
