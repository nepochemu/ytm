use crate::api::YouTubeClient;
use crate::{api, config};
use serde_json::Value;
use std::{
    error::Error,
    fs,
    io::Write,
    process::{Command, Stdio},
};

/// Run fzf on a list of lines, return the selected index (1-based)
fn fzf_select(lines: &[String]) -> Option<usize> {
    let mut child = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start fzf");

    {
        if let Some(mut stdin) = child.stdin.take() {
            for line in lines {
                let _ = writeln!(stdin, "{}", line);
            }
        }
    }

    let output = child.wait_with_output().expect("failed to run fzf");
    let choice = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if choice.is_empty() {
        None
    } else {
        choice.split(':').next().and_then(|s| s.trim().parse().ok())
    }
}

/// Perform a search using YouTubeClient (with cache)
pub async fn search(
    client: &YouTubeClient,
    query: Vec<String>,
    audio_only: bool,
) -> Result<(), Box<dyn Error>> {
    let q = query.join(" ");

    // [!] use client instead of api::search
    let resp = client.search(&q).await?;

    // Save results before fzf so `play()` always has fresh data
    fs::write("last_results.json", serde_json::to_string(&resp)?)?;

    let mut lines = Vec::new();

    let items = resp["items"].as_array().cloned().unwrap_or_default();
    for (i, item) in items.iter().enumerate() {
        let kind = item["id"]["kind"].as_str().unwrap_or("");
        let title = item["snippet"]["title"].as_str().unwrap_or("");

        let line = match kind {
            "youtube#video" => format!("{}: {}", i + 1, title),
            "youtube#playlist" => format!("{}: {} [playlist]", i + 1, title),
            _ => continue,
        };

        lines.push(line);
    }

    if lines.is_empty() {
        println!("No results.");
        return Ok(());
    }

    if let Some(selected) = fzf_select(&lines) {
        return play(client, selected, audio_only).await;
    } else {
        println!("No selection made.");
    }

    Ok(())
}

/// Play a previously selected item
pub async fn play(
    client: &YouTubeClient,
    index: usize,
    audio_only: bool,
) -> Result<(), Box<dyn Error>> {
    let data = match fs::read_to_string("last_results.json") {
        Ok(d) => d,
        Err(_) => {
            eprintln!("No cached results. Run `ytm <query>` first.");
            return Ok(());
        }
    };
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

    let item = &items[index - 1];
    let kind = item["id"]["kind"].as_str().unwrap_or("");

    match kind {
        // Video
        "youtube#video" => {
            let video_id = item["id"]["videoId"].as_str().unwrap_or("");
            if video_id.is_empty() {
                eprintln!("Selected item has no videoId.");
                return Ok(());
            }
            let url = format!("https://youtube.com/watch?v={}", video_id);
            println!("Playing {}", url);

            if audio_only {
                Command::new("mpv").arg("--no-video").arg(&url).status()?;
            } else {
                Command::new("mpv").arg(&url).status()?;
            }
        }

        // Playlist
        "youtube#playlist" => {
            let playlist_id = item["id"]["playlistId"].as_str().unwrap_or("");
            if playlist_id.is_empty() {
                eprintln!("Selected item has no playlistId.");
                return Ok(());
            }
            println!("Fetching playlist {}", playlist_id);

            // [!] use client instead of raw api::fetch_playlist_items
            let resp = client.fetch_playlist_items(playlist_id).await?;

            let empty = Vec::new();
            let videos: Vec<&str> = resp["items"]
                .as_array()
                .unwrap_or(&empty)
                .iter()
                .filter_map(|it| it["snippet"]["resourceId"]["videoId"].as_str())
                .collect();

            if videos.is_empty() {
                eprintln!("No videos found in playlist.");
                return Ok(());
            }

            let urls: Vec<String> = videos
                .iter()
                .map(|id| format!("https://youtube.com/watch?v={}", id))
                .collect();

            println!("Playing playlist with {} videos…", urls.len());

            if audio_only {
                Command::new("mpv").arg("--no-video").args(&urls).status()?;
            } else {
                Command::new("mpv").args(&urls).status()?;
            }
        }

        _ => {
            eprintln!("Unsupported item type");
        }
    }

    Ok(())
}

pub async fn set_api_key(key: String) -> Result<(), Box<dyn Error>> {
    if api::validate_key(&key).await {
        config::save_api_key(&key);
    } else {
        eprintln!("[!] Provided API key is not valid.");
    }
    Ok(())
}
