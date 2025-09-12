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
        _ => false,
    }
}

/// Run a YouTube search (videos + playlists), 30 results
pub async fn search(query: &str, key: &str) -> anyhow::Result<Value> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&type=video,playlist&maxResults=30&q={}&key={}",
        query, key
    );
    let resp = reqwest::get(&url).await?;
    let status = resp.status();
    let text = resp.text().await?;

    if !status.is_success() {
        // Try to surface the API error message nicely
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

