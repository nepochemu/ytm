use crate::{api, config};
use serde_json::Value;
use std::{error::Error, fs, process::Command};

pub async fn search(query: Vec<String>) -> Result<(), Box<dyn Error>> {
    let api_key = config::prompt_api_key(|k| futures::executor::block_on(api::validate_key(k))).await;

    let q = query.join(" ");
    let resp = api::search_videos(&q, &api_key).await?;

    let items: Vec<Value> = resp
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    for (i, item) in items.iter().enumerate() {
        let video_id = item["id"]["videoId"].as_str().unwrap_or("");
        let title = item["snippet"]["title"].as_str().unwrap_or("");
        let channel = item["snippet"]["channelTitle"].as_str().unwrap_or("");
        println!("{}: {} [{}] (watch?v={})", i + 1, title, channel, video_id);
    }

    fs::write("last_results.json", serde_json::to_string(&resp)?)?;
    Ok(())
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

    let video_id = items[index - 1]["id"]["videoId"]
        .as_str()
        .unwrap_or("");
    if video_id.is_empty() {
        eprintln!("Selected item has no videoId.");
        return Ok(());
    }

    let url = format!("https://youtube.com/watch?v={}", video_id);
    println!("Playing {}", url);

    Command::new("mpv")
        .arg(&url)
        .status()
        .expect("failed to launch mpv");

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

