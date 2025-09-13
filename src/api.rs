use reqwest;
use serde_json::Value;
use anyhow::anyhow;

/// Quick check if API key works
pub async fn validate_key(key: &str) -> bool {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&type=video&maxResults=1&q=test&key={}",
        key
    );

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => true,
        Ok(resp) => {
            if let Ok(text) = resp.text().await {
                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                    if let Some(reason) = json["error"]["errors"][0]["reason"].as_str() {
                        eprintln!("❌ API key rejected. Reason: {}", reason);
                    }
                    if let Some(msg) = json["error"]["message"].as_str() {
                        eprintln!("Message: {}", msg);
                    }
                }

                eprintln!("\nPossible reasons:\n\
1. Key is restricted by IP or domain → check restrictions: https://console.cloud.google.com/apis/credentials\n\
2. Daily quota exceeded (10,000 units/day, ~100 searches).\n\
3. YouTube Data API v3 is not enabled → enable here: https://console.cloud.google.com/apis/api/youtube.googleapis.com\n\
4. Key was deleted or rotated in Google Cloud Console.\n\
5. Key entered incorrectly (copy/paste issue).\n");
            }
            false
        }
        Err(e) => {
            eprintln!("❌ Failed to reach YouTube API: {}", e);
            false
        }
    }
}

/// Run a YouTube search (videos + playlists), 50 results
pub async fn search(query: &str, key: &str) -> anyhow::Result<Value> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&type=video,playlist&maxResults=50&q={}&key={}",
        query, key
    );
    let resp = reqwest::get(&url).await?;
    let status = resp.status();
    let text = resp.text().await?;

    if !status.is_success() {
        let msg = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| v.get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .map(|s| s.to_string()))
            .unwrap_or_else(|| text.clone());
        return Err(anyhow!("YouTube API search failed (status {}): {}", status, msg));
    }

    let v: Value = serde_json::from_str(&text)?;
    Ok(v)
}

/// Fetch videos inside a playlist
pub async fn fetch_playlist_items(playlist_id: &str, key: &str) -> anyhow::Result<Value> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&maxResults=50&playlistId={}&key={}",
        playlist_id, key
    );
    let resp = reqwest::get(&url).await?;
    let status = resp.status();
    let text = resp.text().await?;

    if !status.is_success() {
        let msg = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| v.get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .map(|s| s.to_string()))
            .unwrap_or_else(|| text.clone());
        return Err(anyhow!("YouTube API playlistItems failed (status {}): {}", status, msg));
    }

    let v: Value = serde_json::from_str(&text)?;
    Ok(v)
}

