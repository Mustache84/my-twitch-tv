use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use std::fs;
use std::path::PathBuf;

// FIX: Only import keyring in Release Mode
#[cfg(not(debug_assertions))]
use keyring::Entry;

// --- CONFIGURATION ---
const CLIENT_ID: &str = "vrhsf9gxj2y4jntunres6mzber1fg1"; 

// FIX: Only warn about these constants in Release Mode
#[cfg(not(debug_assertions))]
const SERVICE_NAME: &str = "my-twitch-tv-app";
#[cfg(not(debug_assertions))]
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
    println!("[AUTH] Starting Device Flow...");
    let client = Client::new();
    
    let params = [
        ("client_id", CLIENT_ID), 
        ("scopes", "user:read:follows chat:read chat:edit")
    ];

    println!("[AUTH] Sending request to https://id.twitch.tv/oauth2/device...");
    let resp = client.post("https://id.twitch.tv/oauth2/device")
        .form(&params).send().await?;

    let status = resp.status();
    println!("[AUTH] Device Code Status: {}", status);

    if !status.is_success() {
        let err_text = resp.text().await?;
        println!("[AUTH] ERROR: {}", err_text);
        return Err(anyhow!("Failed to initiate device flow: {}", err_text));
    }
    
    let data: DeviceAuthResponse = resp.json().await?;
    println!("[AUTH] Received User Code: {}", data.user_code);
    Ok(data)
}

pub async fn poll_for_token(device_code: String, interval_seconds: u64) -> Result<String> {
    let client = Client::new();
    let mut interval = Duration::from_secs(interval_seconds);

    println!("[AUTH] Starting Polling Loop. Interval: {}s", interval_seconds);

    loop {
        sleep(interval).await;
        
        let params = [
            ("client_id", CLIENT_ID),
            ("device_code", &device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ];

        println!("[AUTH] Polling Twitch...");
        let resp = client.post("https://id.twitch.tv/oauth2/token").form(&params).send().await?;
        let status = resp.status();
        let body = resp.text().await?;

        println!("[AUTH] Poll Status: {} | Body: {}", status, body);

        if status.is_success() {
            println!("[AUTH] 200 OK! Parsing token...");
            let token_data: TokenResponse = serde_json::from_str(&body)?;
            
            save_token_securely(&token_data.access_token)?;
            println!("[AUTH] Token saved successfully.");
            
            return Ok("Authentication Successful".to_string());
        }

        if body.contains("authorization_pending") { 
            println!("[AUTH] User has not authorized yet. Waiting...");
            continue; 
        } 
        else if body.contains("slow_down") { 
            println!("[AUTH] Polling too fast. Increasing interval.");
            interval += Duration::from_secs(5); 
            continue; 
        } 
        else if body.contains("expired_token") { 
            println!("[AUTH] Code expired.");
            return Err(anyhow!("The login code expired. Please try again.")); 
        } 
        else { 
            println!("[AUTH] UNEXPECTED ERROR: {}", body);
            return Err(anyhow!("Auth Error: {}", body)); 
        }
    }
}

// --- STORAGE HELPERS (Dev Switch) ---

fn get_dev_token_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push(".dev_token");
    path
}

pub fn save_token_securely(token: &str) -> Result<()> {
    #[cfg(debug_assertions)]
    {
        println!("[AUTH] Saving token to local file (.dev_token)...");
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
        println!("[AUTH] Reading token from local file...");
        // Handle file not found cleanly
        match fs::read_to_string(path) {
            Ok(token) => Ok(token.trim().to_string()),
            Err(_) => Err(anyhow!("Dev Token file not found"))
        }
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
        println!("[AUTH] Removing local token file...");
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