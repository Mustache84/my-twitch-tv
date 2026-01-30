use anyhow::{anyhow, Result};
use keyring::Entry;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

// --- CONFIGURATION ---
const CLIENT_ID: &str = "vrhsf9gxj2y4jntunres6mzber1fg1"; 
const SERVICE_NAME: &str = "my-twitch-tv-app";
const USER_ACCOUNT: &str = "twitch_access_token";

// --- API STRUCTS ---

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeviceAuthResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ErrorResponse {
    message: String,
}

// --- FLOW IMPLEMENTATION ---

/// Step 1: Ask Twitch for a Device Code
pub async fn start_auth_process() -> Result<DeviceAuthResponse> {
    let client = Client::new();
    
    let params = [
        ("client_id", CLIENT_ID),
        ("scopes", "user:read:follows chat:read chat:edit"),
    ];

    let resp = client.post("https://id.twitch.tv/oauth2/device")
        .form(&params)
        .send()
        .await?;

    if !resp.status().is_success() {
        let err_text = resp.text().await?;
        return Err(anyhow!("Failed to initiate device flow: {}", err_text));
    }

    let auth_data: DeviceAuthResponse = resp.json().await?;
    Ok(auth_data)
}

/// Step 2: Poll Twitch until the user approves
pub async fn poll_for_token(device_code: String, interval_seconds: u64) -> Result<String> {
    let client = Client::new();
    let mut interval = Duration::from_secs(interval_seconds);

    // Poll loop
    loop {
        sleep(interval).await;

        let params = [
            ("client_id", CLIENT_ID),
            ("scopes", "user:read:follows chat:read chat:edit"),
            ("device_code", &device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ];

        let resp = client.post("https://id.twitch.tv/oauth2/token")
            .form(&params)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if status.is_success() {
            // Success! We got the token.
            let token_data: TokenResponse = serde_json::from_str(&body)?;
            
            // Save securely
            save_token_securely(&token_data.access_token)?;
            
            return Ok("Authentication Successful".to_string());
        }

        // Handle specific OAuth errors
        if body.contains("authorization_pending") {
            // User hasn't clicked "Allow" yet. Keep waiting.
            continue; 
        } else if body.contains("slow_down") {
            // We are polling too fast. Add 5 seconds.
            interval += Duration::from_secs(5);
            continue;
        } else if body.contains("expired_token") {
            return Err(anyhow!("The login code expired. Please try again."));
        } else {
            // Unknown error (denied, invalid_client, etc.)
            return Err(anyhow!("Auth Error: {}", body));
        }
    }
}

// --- SECURE STORAGE ---

pub fn save_token_securely(token: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, USER_ACCOUNT)?;
    entry.set_password(token)?;
    Ok(())
}

pub fn get_cached_token() -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, USER_ACCOUNT)?;
    let token = entry.get_password()?;
    Ok(token)
}

pub fn logout() -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, USER_ACCOUNT)?;
    match entry.delete_password() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(anyhow!(e)),
    }
}