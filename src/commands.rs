use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::api::{SearchItem, YouTubeClient};
use crate::cache::Cache;
use crate::mpv::{self, Mpv};

/// Get the directory for application cache
fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ytm")
}

/// Format seconds as MM:SS string
fn format_time(seconds: Option<f64>) -> String {
    match seconds {
        Some(secs) => {
            let minutes = (secs / 60.0).floor() as u32;
            let seconds = (secs % 60.0).round() as u32;
            format!("{:02}:{:02}", minutes, seconds)
        }
        None => "--:--".to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SearchChoice {
    label: String,
    target: SearchTarget,
}

impl SearchChoice {
    fn from_item(item: &SearchItem) -> Option<Self> {
        let title = item.snippet.title.trim();
        let channel = item.snippet.channel_title.trim();

        if let Some(id) = item.id.video_id.as_ref() {
            let label = format!("{} | {}", title, channel);
            return Some(Self {
                label,
                target: SearchTarget::Video(id.clone()),
            });
        }

        item.id.playlist_id.as_ref().map(|id| Self {
            label: format!("{} | {} [playlist]", title, channel),
            target: SearchTarget::Playlist(id.clone()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SearchTarget {
    Video(String),
    Playlist(String),
}

impl SearchTarget {
    fn url(&self) -> String {
        match self {
            SearchTarget::Video(id) => format!("https://www.youtube.com/watch?v={id}"),
            SearchTarget::Playlist(id) => format!("https://www.youtube.com/playlist?list={id}"),
        }
    }
}

/// Simple status display
fn show_detailed_status() -> anyhow::Result<()> {
    std::thread::sleep(std::time::Duration::from_secs(1));

    if let Ok(mut mpv_client) = Mpv::connect() {
        if let Ok(status) = mpv_client.get_status() {
            if let Some(title) = status.title {
                println!("{}", title);

                // Show album or playlist title
                if let Some(album) = &status.album {
                    println!("Album: {}", album);
                } else if let Some(playlist) = &status.playlist_title {
                    println!("Playlist: {}", playlist);
                }

                let pos_str = format_time(status.position);
                let dur_str = format_time(status.duration);
                let percentage = if let (Some(pos), Some(dur)) = (status.position, status.duration)
                {
                    if dur > 0.0 {
                        (pos / dur * 100.0) as u32
                    } else {
                        0
                    }
                } else {
                    0
                };

                // Format playlist info
                let playlist_info = if let (Some(pos), Some(count)) =
                    (status.playlist_pos, status.playlist_count)
                {
                    if count > 1 {
                        format!("[playlist] #{}/{}", pos, count)
                    } else {
                        "[playing]".to_string()
                    }
                } else {
                    "[playing]".to_string()
                };

                println!(
                    "{}   {} / {} ({}%)",
                    playlist_info, pos_str, dur_str, percentage
                );
                println!("volume: 100%");
                return Ok(());
            }
        }
    }

    println!("Player not responding");
    Ok(())
}

/// Search YouTube, pick first result, and play
pub async fn search_and_play(query: &str, no_video: bool, background: bool) -> anyhow::Result<()> {
    let cache_root = cache_dir();
    let cache = Cache::new(&cache_root, std::time::Duration::from_secs(3600))?;
    let client = YouTubeClient::new(&cache_root, cache)?;

    // Fetch 50 results
    let results = client.search(query, Some(50)).await?;
    if results.is_empty() {
        return Err(anyhow::anyhow!("No results for '{}'", query));
    }

    let choices: Vec<SearchChoice> =
        results.iter().filter_map(SearchChoice::from_item).collect();

    if choices.is_empty() {
        return Err(anyhow::anyhow!(
            "No playable videos or playlists found for '{}'",
            query
        ));
    }

    let fzf = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        let mut stdin = fzf.stdin.as_ref().unwrap();
        for choice in &choices {
            writeln!(stdin, "{}", choice.label)?;
        }
    }
    let output = fzf.wait_with_output()?;
    if !output.status.success() {
        return Ok(());
    }
    let selected = String::from_utf8_lossy(&output.stdout);
    if selected.trim().is_empty() {
        return Ok(());
    }
    let selected_line = selected.trim();
    let choice = choices
        .into_iter()
        .find(|candidate| candidate.label == selected_line)
        .ok_or_else(|| anyhow::anyhow!("Selection not found"))?;

    play(&choice.target.url(), no_video, background)
}

/// Start mpv either foreground or background with IPC enabled
pub fn play(url: &str, no_video: bool, background: bool) -> anyhow::Result<()> {
    let mut args = Vec::new();
    if no_video {
        args.push("--no-video");
    }
    if background {
        args.push("--input-ipc-server=/tmp/ytm-mpv.sock");
    }
    args.push(url);

    if background {
        // Start mpv in background with output suppressed from the start
        let _child = Command::new("mpv")
            .args(&args)
            .stdout(std::process::Stdio::null()) // Suppress stdout immediately
            .stderr(std::process::Stdio::null()) // Suppress stderr immediately
            .stdin(std::process::Stdio::null()) // Also suppress stdin
            .spawn()?;

        // Show status with retry logic
        show_detailed_status()?;
        println!("\nPlayer started in background. Use 'ytm stop/pause/next/prev' to control.");

        Ok(())
    } else {
        // Run in foreground (blocking)
        Command::new("mpv").args(&args).status()?;
        Ok(())
    }
}

pub fn pause() -> anyhow::Result<()> {
    use serde_json::json;
    mpv::send_mpv_command(json!({"command": ["cycle", "pause"]}))
}

pub fn resume() -> anyhow::Result<()> {
    use serde_json::json;
    mpv::send_mpv_command(json!({"command": ["cycle", "pause"]}))
}

pub fn next() -> anyhow::Result<()> {
    use serde_json::json;

    // Get current track position before change
    let current_pos = if let Ok(mut client) = Mpv::connect() {
        client
            .get_property("playlist-pos-1")
            .ok()
            .flatten()
            .and_then(|v| v.as_i64())
    } else {
        None
    };

    mpv::send_mpv_command(json!({"command": ["playlist-next", "force"]}))?;

    // Wait for actual track position change (not timing)
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(200));
        if let Ok(mut client) = Mpv::connect() {
            if let Ok(Some(new_pos)) = client
                .get_property("playlist-pos-1")
                .map(|v| v.and_then(|val| val.as_i64()))
            {
                if Some(new_pos) != current_pos {
                    // Position changed, show status
                    show_detailed_status()?;
                    return Ok(());
                }
            }
        }
    }

    // Fallback if position detection fails
    show_detailed_status()?;
    Ok(())
}

pub fn prev() -> anyhow::Result<()> {
    use serde_json::json;

    // Get current track position before change
    let current_pos = if let Ok(mut client) = Mpv::connect() {
        client
            .get_property("playlist-pos-1")
            .ok()
            .flatten()
            .and_then(|v| v.as_i64())
    } else {
        None
    };

    mpv::send_mpv_command(json!({"command": ["playlist-prev", "force"]}))?;

    // Wait for actual track position change (not timing)
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(200));
        if let Ok(mut client) = Mpv::connect() {
            if let Ok(Some(new_pos)) = client
                .get_property("playlist-pos-1")
                .map(|v| v.and_then(|val| val.as_i64()))
            {
                if Some(new_pos) != current_pos {
                    // Position changed, show status
                    show_detailed_status()?;
                    return Ok(());
                }
            }
        }
    }

    // Fallback if position detection fails
    show_detailed_status()?;
    Ok(())
}

pub fn stop() -> anyhow::Result<()> {
    use serde_json::json;
    if mpv::send_mpv_command(json!({"command": ["stop"]})).is_ok() {
        return Ok(());
    }
    // Fallback to force kill
    mpv::force_kill()
}

pub fn status() -> anyhow::Result<()> {
    if !mpv::is_running() {
        println!("No player currently running");
        return Ok(());
    }

    show_detailed_status()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time() {
        assert_eq!(format_time(Some(0.0)), "00:00");
        assert_eq!(format_time(Some(59.0)), "00:59");
        assert_eq!(format_time(Some(60.0)), "01:00");
        assert_eq!(format_time(Some(125.5)), "02:06");
        assert_eq!(format_time(None), "--:--");
    }
}
