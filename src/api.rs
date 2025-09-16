use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cache::Cache;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Video {
    pub id: serde_json::Value,
    pub snippet: serde_json::Value,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct VideoId {
    #[serde(rename = "kind")]
    pub kind: String,
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
    #[serde(rename = "playlistId")]
    pub playlist_id: Option<String>,
}

pub struct YouTubeClient {
    api_key: String,
    http: Client,
    cache: Cache,
}

impl YouTubeClient {
    pub fn new(api_key: String, cache: Cache) -> Self {
        Self {
            api_key,
            http: Client::new(),
            cache,
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Video>> {
        let url = format!(
            "https://www.googleapis.com/youtube/v3/search?part=snippet&q={}&type=video&maxResults=5&key={}",
            query, self.api_key
        );

        if let Some(cached) = self.cache.get::<Vec<Video>>(&url) {
            return Ok(cached);
        }

        let resp: Value = self.http.get(&url).send().await?.json().await?;
        let items: Vec<Video> = match serde_json::from_value(resp["items"].clone()) {
            Ok(items) => items,
            Err(e) => {
                return Err(e.into());
            }
        };

        // Only keep videos (with videoId)
        let videos = items
            .into_iter()
            .filter(|v| v.id.get("videoId").is_some() || v.id.get("playlistId").is_some())
            .collect::<Vec<_>>();

        self.cache.put(&url, &videos)?;
        Ok(videos)
    }

    pub async fn search_with_max_results(
        &self,
        query: &str,
        max_results: u32,
    ) -> anyhow::Result<Vec<Video>> {
        let url = format!(
            "https://www.googleapis.com/youtube/v3/search?part=snippet&q={}&type=video,playlist&maxResults={}&key={}",
            query, max_results, self.api_key
        );

        if let Some(cached) = self.cache.get::<Vec<Video>>(&url) {
            return Ok(cached);
        }

        let resp: serde_json::Value = self.http.get(&url).send().await?.json().await?;
        let items: Vec<Video> = match serde_json::from_value(resp["items"].clone()) {
            Ok(items) => items,
            Err(e) => {
                return Err(e.into());
            }
        };

        let videos = items
            .into_iter()
            .filter(|v| v.id.get("videoId").is_some() || v.id.get("playlistId").is_some())
            .collect::<Vec<_>>();

        self.cache.put(&url, &videos)?;
        Ok(videos)
    }
}

/// 🔑 Quick check if API key works
pub async fn validate_key(key: &str) -> bool {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&q=test&type=video&maxResults=1&key={}",
        key
    );

    match Client::new().get(&url).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}
