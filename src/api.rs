use crate::cache::Cache;
use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct VideoMeta {
    pub id: String,
    pub title: String,
    pub channel: String,
    pub url: String,
}

pub struct YouTubeClient {
    api_key: String,
    cache: Cache,
}

impl YouTubeClient {
    pub fn new(api_key: String, cache: Cache) -> Self {
        Self { api_key, cache }
    }

    /// Search YouTube for a query string
    pub async fn search(&self, query: &str) -> Result<Value> {
        if let Some(cached) = self.cache.get::<Value>(query) {
            eprintln!("⚡ Using cached results for: {}", query);
            return Ok(cached);
        }

        let url = "https://www.googleapis.com/youtube/v3/search";
        let resp: Value = reqwest::Client::new()
            .get(url)
            .query(&[
                ("part", "snippet"),
                ("type", "video,playlist"),
                ("maxResults", "50"),
                ("q", query),
                ("key", &self.api_key),
            ])
            .send()
            .await?
            .json()
            .await?;

        self.cache.put(query, &resp)?;
        eprintln!("[i] Cached results for: {}", query);

        Ok(resp)
    }

    /// Fetch playlist items by playlistId
    pub async fn fetch_playlist_items(&self, playlist_id: &str) -> Result<Value> {
        let key = format!("playlist:{}", playlist_id);
        if let Some(cached) = self.cache.get::<Value>(&key) {
            eprintln!("[i] Using cached playlist: {}", playlist_id);
            return Ok(cached);
        }

        let url = "https://www.googleapis.com/youtube/v3/playlistItems";
        let resp: Value = reqwest::Client::new()
            .get(url)
            .query(&[
                ("part", "snippet"),
                ("maxResults", "50"),
                ("playlistId", playlist_id),
                ("key", &self.api_key),
            ])
            .send()
            .await?
            .json()
            .await?;

        self.cache.put(&key, &resp)?;
        eprintln!("[i] Cached playlist: {}", playlist_id);

        Ok(resp)
    }
}

// ✅ validate_key is NOT tied to a client (it takes `key` directly)
pub async fn validate_key(key: &str) -> bool {
    let url = "https://www.googleapis.com/youtube/v3/search";
    let resp = reqwest::Client::new()
        .get(url)
        .query(&[
            ("part", "snippet"),
            ("maxResults", "1"),
            ("q", "test"),
            ("key", key),
        ])
        .send()
        .await;

    resp.map(|r| r.status().is_success()).unwrap_or(false)
}
