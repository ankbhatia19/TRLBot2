use reqwest::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::multipart;
use serde_json::{json, Value};
use std::{fs, time::Duration};
use tokio::sync::Semaphore;
use tokio::time::{sleep, Instant};
use once_cell::sync::Lazy;

// Initialize the HTTP client once and make it globally accessible
static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .build()
        .expect("Failed to build HTTP client")
});

// Semaphore to manage rate limiting, allowing up to 4 requests per second
static SEMAPHORE: Lazy<Semaphore> = Lazy::new(|| Semaphore::new(4));

// Helper function to enforce rate limiting
async fn rate_limited() {
    let permit = SEMAPHORE.acquire().await.unwrap();
    tokio::spawn(async move {
        // Drop permit after 1 second to allow the next batch
        sleep(Duration::from_secs(1)).await;
        drop(permit);
    });
}

// Function to upload a replay file
pub async fn upload(path: &str, replay_name: &str) -> Result<Value, reqwest::Error> {
    rate_limited().await;
    let token: String = std::env::var("BALLCHASING_TOKEN")
        .expect("Ballchasing Token is required.");
    let file_content = fs::read(path).expect("Failed to read file");

    let form = multipart::Form::new()
        .part("file", multipart::Part::bytes(file_content)
            .file_name(replay_name.to_string())
            .mime_str("multipart/form-data")?
        );

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());

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
    rate_limited().await;

    let token: String = std::env::var("BALLCHASING_TOKEN")
        .expect("Ballchasing Token is required.");

    let group_id:String = std::env::var("BALLCHASING_GROUP")
        .expect("Ballchasing Group is required.");

    let post_body = json!({
        "name": match_id.to_string(),
        "parent": group_id,
        "player_identification": "by-id",
        "team_identification": "by-player-clusters"
    });

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());

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
    rate_limited().await;

    let token: String = std::env::var("BALLCHASING_TOKEN")
        .expect("Ballchasing Token is required.");

    let patch_body = json!({
        "title": replay_name,
        "group": group
    });

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());

    CLIENT
        .patch(&format!("https://ballchasing.com/api/replays/{}", ballchasing_id))
        .headers(headers)
        .json(&patch_body)
        .send()
        .await?;

    Ok(())
}

// Function to remove a replay from its group
pub async fn ungroup(ballchasing_id: &str) -> Result<(), reqwest::Error> {
    rate_limited().await;

    let token: String = std::env::var("BALLCHASING_TOKEN")
        .expect("Ballchasing Token is required.");

    let patch_body = json!({
        "group": ""
    });

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());

    CLIENT
        .patch(&format!("https://ballchasing.com/api/replays/{}", ballchasing_id))
        .headers(headers)
        .json(&patch_body)
        .send()
        .await?;

    Ok(())
}

pub async fn delete_group(ballchasing_id: &str) -> Result<(), reqwest::Error> {
    rate_limited().await;

    let token: String = std::env::var("BALLCHASING_TOKEN")
        .expect("Ballchasing Token is required.");

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());

    CLIENT
        .delete(&format!("https://ballchasing.com/api/groups/{}", ballchasing_id))
        .headers(headers)
        .send()
        .await?;

    Ok(())
}

// Function to pull a replay's status, retrying if it is still pending
pub async fn pull(ballchasing_id: &str) -> Result<Value, reqwest::Error> {
    let get_endpoint = format!("https://ballchasing.com/api/replays/{}", ballchasing_id);

    let token: String = std::env::var("BALLCHASING_TOKEN")
        .expect("Ballchasing Token is required.");

    loop {
        rate_limited().await;

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());

        let response: Response = CLIENT
            .get(&get_endpoint)
            .headers(headers)
            .send()
            .await?;

        let replay_data = response.json::<Value>().await?;

        if replay_data["status"].as_str() != Some("pending") {
            return Ok(replay_data);
        }

        sleep(Duration::from_millis(500)).await;  // Wait before retrying
    }
}
