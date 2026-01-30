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
    let token = get_cached_token()?;
    let client = Client::new();

    // 1. Get My User ID
    let user_resp = client
        .get("https://api.twitch.tv/helix/users")
        .header("Client-Id", CLIENT_ID)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?
        .error_for_status()?;

    let user_data: UserResponse = user_resp.json().await?;
    let my_id = user_data.data.first().ok_or(anyhow!("No user found"))?.id.clone();

    // 2. Get Followed Streams
    let streams_resp = client
        .get("https://api.twitch.tv/helix/streams/followed")
        .query(&[("user_id", my_id)])
        .header("Client-Id", CLIENT_ID)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?
        .error_for_status()?;

    let streams_data: StreamsResponse = streams_resp.json().await?;
    
    Ok(streams_data.data)
}