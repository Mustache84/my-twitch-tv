use anyhow::{anyhow, Result};
use keyring::Entry;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use std::fs;
use std::path::PathBuf;

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
}

// --- FLOW IMPLEMENTATION ---

pub async fn start_auth_process() -> Result<DeviceAuthResponse> {
    let client = Client::new();
    // STEP 1: Scopes are required here
    let params = [("client_id", CLIENT_ID), ("scopes", "user:read:follows chat:read chat:edit")];

    println!("Requesting Device Code from Twitch..."); // <--- LOGGING

    let resp = client.post("https://id.twitch.tv/oauth2/device")
        .form(&params).send().await?;

    if !resp.status().is_success() {
        let error_msg = resp.text().await?;
        println!("Error getting code: {}", error_msg); // <--- LOGGING
        return Err(anyhow!("Failed to initiate device flow: {}", error_msg));
    }
    
    let data: DeviceAuthResponse = resp.json().await?;
    println!("Device Code Received. User Code: {}", data.user_code); // <--- LOGGING
    Ok(data)
}

pub async fn poll_for_token(device_code: String, interval_seconds: u64) -> Result<String> {
    let client = Client::new();
    let mut interval = Duration::from_secs(interval_seconds);

    println!("Starting Polling Loop. Interval: {}s", interval_seconds);

    loop {
        sleep(interval).await;
        
        // STEP 2: Scopes should NOT be sent here (only client_id, device_code, grant_type)
        let params = [
            ("client_id", CLIENT_ID),
            ("device_code", &device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ];

        let resp = client.post("https://id.twitch.tv/oauth2/token").form(&params).send().await?;
        let status = resp.status();
        let body = resp.text().await?;

        // LOGGING: This will show you exactly what Twitch is saying
        println!("Poll Status: {} | Body: {}", status, body); 

        if status.is_success() {
            let token_data: TokenResponse = serde_json::from_str(&body)?;
            save_token_securely(&token_data.access_token)?;
            println!("LOGIN SUCCESS! Token saved.");
            return Ok("Authentication Successful".to_string());
        }

        if body.contains("authorization_pending") { 
            // Normal behavior, waiting for user
            continue; 
        } 
        else if body.contains("slow_down") { 
            println!("Twitch says slow down. Adding 5 seconds.");
            interval += Duration::from_secs(5); 
            continue; 
        } 
        else if body.contains("expired_token") { 
            return Err(anyhow!("The login code expired. Please try again.")); 
        } 
        else { 
            // If we get here, it's a real error (like 400 Bad Request)
            return Err(anyhow!("Auth Error: {}", body)); 
        }
    }
}

// --- STORAGE HELPERS ---

fn get_dev_token_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push(".dev_token");
    path
}

pub fn save_token_securely(token: &str) -> Result<()> {
    #[cfg(debug_assertions)]
    {
        let path = get_dev_token_path();
        fs::write(path, token)?;
        return Ok(());
    }
    #[cfg(not(debug_assertions))]
    {
        let entry = Entry::new(SERVICE_NAME, USER_ACCOUNT)?;
        entry.set_password(token)?;
        Ok(())
    }
}

pub fn get_cached_token() -> Result<String> {
    #[cfg(debug_assertions)]
    {
        let path = get_dev_token_path();
        let token = fs::read_to_string(path)?;
        return Ok(token.trim().to_string());
    }
    #[cfg(not(debug_assertions))]
    {
        let entry = Entry::new(SERVICE_NAME, USER_ACCOUNT)?;
        let token = entry.get_password()?;
        Ok(token)
    }
}

pub fn logout() -> Result<()> {
    #[cfg(debug_assertions)]
    {
        let path = get_dev_token_path();
        if path.exists() { fs::remove_file(path)?; }
        Ok(())
    }
    #[cfg(not(debug_assertions))]
    {
        let entry = Entry::new(SERVICE_NAME, USER_ACCOUNT)?;
        let _ = entry.delete_password();
        Ok(())
    }
}