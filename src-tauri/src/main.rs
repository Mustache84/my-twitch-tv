#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod auth;
mod api; 

// REMOVED: use tauri::Manager; (Fixes warning)

// --- AUTH COMMANDS ---
#[tauri::command]
async fn start_login_command() -> Result<auth::oauth::DeviceAuthResponse, String> {
    println!("[CMD] start_login_command invoked");
    auth::oauth::start_auth_process().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn finish_login_command(device_code: String, interval: u64) -> Result<String, String> {
    println!("[CMD] finish_login_command invoked. Code: {}", device_code);
    auth::oauth::poll_for_token(device_code, interval).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn logout_command() -> Result<(), String> {
    println!("[CMD] logout_command invoked");
    auth::oauth::logout().map_err(|e| e.to_string())
}

#[tauri::command]
fn check_auth_command() -> bool {
    // ADDED: Logging so you can see it working
    match auth::oauth::get_cached_token() {
        Ok(_) => {
            println!("[CMD] check_auth_command: Token FOUND. Auto-login.");
            true
        },
        Err(_) => {
            println!("[CMD] check_auth_command: Token NOT found.");
            false
        }
    }
}

// --- DATA COMMANDS ---
#[tauri::command]
async fn get_live_channels_command() -> Result<Vec<api::twitch::Stream>, String> {
    println!("[CMD] get_live_channels_command invoked");
    api::twitch::get_live_followed_channels()
        .await
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_login_command,
            finish_login_command,
            logout_command,
            check_auth_command,
            get_live_channels_command 
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}