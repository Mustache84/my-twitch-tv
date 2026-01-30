use crate::auth::oauth::get_cached_token;
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const CLIENT_ID: &str = "vrhsf9gxj2y4jntunres6mzber1fg1"; 

// --- DATA STRUCTURES ---

#[derive(Deserialize, Debug)]
struct UserResponse {
    data: Vec<UserData>,
}

#[derive(Deserialize, Debug)]
struct UserData {
    id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Stream {
    pub id: String,
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub game_name: String,
    pub title: String,
    pub viewer_count: u32,
    pub started_at: String,
    pub thumbnail_url: String,
}

#[derive(Deserialize, Debug)]
struct StreamsResponse {
    data: Vec<Stream>,
}

// --- LOGIC ---

pub async fn get_live_followed_channels() -> Result<Vec<Stream>> {
    println!("[API] Fetching live channels...");
    
    let token = get_cached_token().map_err(|_| anyhow!("Not Logged In"))?;
    let client = Client::new();

    // 1. Get My User ID
    println!("[API] Requesting User ID...");
    let user_resp = client
        .get("https://api.twitch.tv/helix/users")
        .header("Client-Id", CLIENT_ID)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if !user_resp.status().is_success() {
        let err = user_resp.text().await?;
        println!("[API] Failed to get User ID: {}", err);
        return Err(anyhow!("Twitch API Error: {}", err));
    }

    let user_data: UserResponse = user_resp.json().await?;
    let my_id = user_data.data.first().ok_or(anyhow!("No user found"))?.id.clone();
    println!("[API] User ID Found: {}", my_id);

    // 2. Get Followed Streams
    println!("[API] Requesting Followed Streams...");
    let streams_resp = client
        .get("https://api.twitch.tv/helix/streams/followed")
        .query(&[("user_id", my_id)])
        .header("Client-Id", CLIENT_ID)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if !streams_resp.status().is_success() {
        let err = streams_resp.text().await?;
        println!("[API] Failed to get Streams: {}", err);
        return Err(anyhow!("Twitch Stream Error: {}", err));
    }

    let streams_data: StreamsResponse = streams_resp.json().await?;
    println!("[API] Success! Found {} live streams.", streams_data.data.len());
    
    Ok(streams_data.data)
}