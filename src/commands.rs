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
        let mut stdin = child.stdin.take().unwrap();
        for line in lines {
            writeln!(stdin, "{}", line).unwrap();
        }
    }

    let output = child.wait_with_output().expect("failed to run fzf");
    let choice = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if choice.is_empty() {
        None
    } else {
        choice.split(':').next()?.trim().parse().ok()
    }
}

pub async fn search(query: Vec<String>) -> Result<(), Box<dyn Error>> {
    let q = query.join(" ");

    // Load or prompt for a key
    let api_key = config::load_or_prompt_api_key().await;

    // Try search; on API error, prompt for a fresh key and retry once
    let resp = match api::search(&q, &api_key).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("⚠️  YouTube API error: {e}");
            println!("Let's update your API key.");
            let new_key = config::prompt_api_key().await;
            api::search(&q, &new_key).await?
        }
    };

    let items: Vec<Value> = resp
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // ✅ Write results BEFORE fzf so `play()` always has fresh data
    fs::write("last_results.json", serde_json::to_string(&resp)?)?;

    // Build fzf lines
    let mut lines = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let kind = item["id"]["kind"].as_str().unwrap_or("");
        let title = item["snippet"]["title"].as_str().unwrap_or("");
        let channel = item["snippet"]["channelTitle"].as_str().unwrap_or("");

        let line = match kind {
            "youtube#video" => format!("{}: {} [{}]", i + 1, title, channel),
            "youtube#playlist" => format!("{}: {} [{}] [playlist]", i + 1, title, channel),
            _ => continue,
        };
        lines.push(line);
    }

    if lines.is_empty() {
        println!("No results.");
        return Ok(());
    }

    if let Some(selected) = fzf_select(&lines) {
        return play(selected).await;
    } else {
        println!("No selection made.");
        return Ok(());
    }
}


pub async fn play(index: usize) -> Result<(), Box<dyn Error>> {
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

    let item = &items[index - 1];
    let kind = item["id"]["kind"].as_str().unwrap_or("");

    match kind {
        // 🎥 Video
        "youtube#video" => {
            let video_id = item["id"]["videoId"].as_str().unwrap_or("");
            if video_id.is_empty() {
                eprintln!("Selected item has no videoId.");
                return Ok(());
            }
            let url = format!("https://youtube.com/watch?v={}", video_id);
            println!("Playing {}", url);

            Command::new("mpv").arg(&url).status()?;
        }

        // 📃 Playlist
        "youtube#playlist" => {
            let playlist_id = item["id"]["playlistId"].as_str().unwrap_or("");
            if playlist_id.is_empty() {
                eprintln!("Selected item has no playlistId.");
                return Ok(());
            }
            println!("Fetching playlist {}", playlist_id);

            let api_key = config::load_or_prompt_api_key().await;
            let resp = api::fetch_playlist_items(playlist_id, &api_key).await?;

            // ✅ Fix: give empty Vec a name so lifetime is long enough
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

            Command::new("mpv").args(&urls).status()?;
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
        eprintln!("❌ Provided API key is not valid.");
    }
    Ok(())
}

