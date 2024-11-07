use reqwest::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::multipart;
use serde_json::{json, Value};
use std::{fs, time::Duration};
use tokio::time::sleep;
use once_cell::sync::Lazy;

// Constants for token and group_id (assuming they are static and set once)
static TOKEN: &str = env!("BALLCHASING_TOKEN");
static GROUP_ID: &str = env!("BALLCHASING_GROUP");

// Initialize the HTTP client once and make it globally accessible
static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .build()
        .expect("Failed to build HTTP client")
});

// Function to upload a replay file
pub async fn upload(path: &str, replay_name: &str) -> Result<Value, reqwest::Error> {
    let file_content = fs::read(path).expect("Failed to read file");

    let form = multipart::Form::new()
        .part("file", multipart::Part::bytes(file_content)
            .file_name(replay_name.to_string())
            .mime_str("multipart/form-data")?
        );

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(TOKEN).unwrap());

    let response = CLIENT
        .post("https://ballchasing.com/api/v2/upload?visibility=public")
        .headers(headers)
        .multipart(form)
        .send()
        .await?;

    response.json().await
}

// Function to create a new subgroup for a match    
pub async fn create(match_id: i32) -> Result<Value, reqwest::Error> {
    let post_body = json!({
        "name": match_id.to_string(),
        "parent": GROUP_ID,
        "player_identification": "by-id",
        "team_identification": "by-player-clusters"
    });

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(TOKEN).unwrap());

    let response = CLIENT
        .post("https://ballchasing.com/api/groups")
        .headers(headers)
        .json(&post_body)
        .send()
        .await?;

    response.json().await
}

// Function to group a replay under a specific group
pub async fn group(replay_name: &str, group: &str, ballchasing_id: &str) -> Result<(), reqwest::Error> {
    let patch_body = json!({
        "title": replay_name,
        "group": group
    });

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(TOKEN).unwrap());

    CLIENT
        .patch(&format!("https://ballchasing.com/api/replays/{}", ballchasing_id))
        .headers(headers)
        .json(&patch_body)
        .send()
        .await?;

    Ok(())
}

// Function to pull a replay's status, retrying if it is still pending
pub async fn pull(ballchasing_id: &str) -> Result<Value, reqwest::Error> {
    let get_endpoint = format!("https://ballchasing.com/api/replays/{}", ballchasing_id);
    let mut replay_data;

    loop {
        sleep(Duration::from_millis(500)).await;

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(TOKEN).unwrap());

        let response: Response = CLIENT
            .get(&get_endpoint)
            .headers(headers)
            .send()
            .await?;

        replay_data = response.json::<Value>().await?;

        if replay_data["status"].as_str() != Some("pending") {
            break;
        }
    }

    Ok(replay_data)
}