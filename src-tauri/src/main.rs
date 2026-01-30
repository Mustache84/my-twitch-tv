#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod auth;
use tauri::Manager;

// --- COMMAND DEFINITIONS ---

// 1. Start: Returns the code to show the user
#[tauri::command]
async fn start_login_command() -> Result<auth::oauth::DeviceAuthResponse, String> {
  auth::oauth::start_auth_process()
      .await
      .map_err(|e| e.to_string())
}

// 2. Poll: Waits for the user to click "Confirm" on the web
#[tauri::command]
async fn finish_login_command(device_code: String, interval: u64) -> Result<String, String> {
  auth::oauth::poll_for_token(device_code, interval)
      .await
      .map_err(|e| e.to_string())
}

#[tauri::command]
fn logout_command() -> Result<(), String> {
  auth::oauth::logout().map_err(|e| e.to_string())
}

#[tauri::command]
fn check_auth_command() -> bool {
  auth::oauth::get_cached_token().is_ok()
}

// --- APP ENTRY POINT ---

fn main() {
  tauri::Builder::default()
      // THIS LINE IS CRITICAL: It registers the function names for JS
      .invoke_handler(tauri::generate_handler![
          start_login_command,   // <--- Must match JS invoke('start_login_command')
          finish_login_command,  // <--- Must match JS invoke('finish_login_command')
          logout_command,
          check_auth_command
      ])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}