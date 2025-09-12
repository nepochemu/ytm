use reqwest;
use serde_json::Value;

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

/// Run a YouTube search (videos + playlists)
pub async fn search(query: &str, key: &str) -> anyhow::Result<Value> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&type=video,playlist&maxResults=20&q={}&key={}",
        query, key
    );
    let resp_text = reqwest::get(&url).await?.text().await?;
    let resp: Value = serde_json::from_str(&resp_text)?;
    Ok(resp)
}

/// Fetch videos inside a playlist
pub async fn fetch_playlist_items(playlist_id: &str, key: &str) -> anyhow::Result<Value> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&maxResults=50&playlistId={}&key={}",
        playlist_id, key
    );
    let resp_text = reqwest::get(&url).await?.text().await?;
    let resp: Value = serde_json::from_str(&resp_text)?;
    Ok(resp)
}

