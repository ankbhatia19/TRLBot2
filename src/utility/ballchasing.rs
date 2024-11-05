use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::blocking::multipart;
use serde_json::{json, Value};
use std::fs;
use std::thread::sleep;
use std::time::Duration;

// Constants for token and group_id (assuming they are static and set once)
static TOKEN: &str = env!("BALLCHASING_TOKEN");
static GROUP_ID: &str = env!("BALLCHASING_GROUP");

// Initialize the HTTP client once
fn get_client() -> Client {
    Client::builder()
        .danger_accept_invalid_certs(false)
        .build()
        .expect("Failed to build HTTP client")
}

// Function to upload a replay file
pub fn upload(path: &str, replay_name: &str) -> Value {
    let client = get_client();
    let file_content = fs::read(path).expect("Failed to read file");

    let form = multipart::Form::new()
        .part("file", multipart::Part::bytes(file_content)
            .file_name(replay_name.to_string())
            .mime_str("multipart/form-data").unwrap());

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(TOKEN).unwrap());

    let response = client
        .post("https://ballchasing.com/api/v2/upload?visibility=public")
        .headers(headers)
        .multipart(form)
        .send()
        .expect("Failed to upload");

    response.json().expect("Failed to parse response as JSON")
}

// Function to create a new subgroup for a match
pub fn create(match_id: i32) -> Value {
    let client = get_client();
    let post_body = json!({
        "name": match_id.to_string(),
        "parent": GROUP_ID,
        "player_identification": "by-id",
        "team_identification": "by-player-clusters"
    });

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(TOKEN).unwrap());

    let response = client
        .post("https://ballchasing.com/api/groups")
        .headers(headers)
        .json(&post_body)
        .send()
        .expect("Failed to create group");

    response.json().expect("Failed to parse response as JSON")
}

// Function to group a replay under a specific group
pub fn group(replay_name: &str, group: &str, ballchasing_id: &str) {
    let client = get_client();
    let patch_body = json!({
        "title": replay_name,
        "group": group
    });

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(TOKEN).unwrap());

    client
        .patch(&format!("https://ballchasing.com/api/replays/{}", ballchasing_id))
        .headers(headers)
        .json(&patch_body)
        .send()
        .expect("Failed to patch group");
}

// Function to pull a replay's status, retrying if it is still pending
pub fn pull(ballchasing_id: &str) -> Value {
    let client = get_client();
    let get_endpoint = format!("https://ballchasing.com/api/replays/{}", ballchasing_id);
    let mut replay_data;

    loop {
        sleep(Duration::from_millis(500));

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(TOKEN).unwrap());

        let response: Response = client
            .get(&get_endpoint)
            .headers(headers)
            .send()
            .expect("Failed to pull replay");

        replay_data = response.json::<Value>().expect("Failed to parse response as JSON");

        if replay_data["status"].as_str() != Some("pending") {
            break;
        }
    }

    replay_data
}
