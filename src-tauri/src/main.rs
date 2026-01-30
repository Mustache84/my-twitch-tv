#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

// 1. Register modules
mod auth;
// mod api;    // We will enable these later
// mod player; // We will enable these later

use tauri::Manager;

// 2. Define Command Wrappers
// These functions are the "Bridge" that JS calls. 
// They execute our heavy Rust logic and return results.

#[tauri::command]
async fn login_command() -> Result<String, String> {
  // Calls the complex PKCE flow we wrote in auth/oauth.rs
  // .map_err converts the Rust error into a String the JS frontend can read
  auth::oauth::login_flow()
      .await
      .map_err(|e| e.to_string())
}

#[tauri::command]
fn logout_command() -> Result<(), String> {
  auth::oauth::logout()
      .map_err(|e| e.to_string())
}

#[tauri::command]
fn check_auth_command() -> bool {
  // Simple check: Do we have a token in the secure vault?
  auth::oauth::get_cached_token().is_ok()
}

fn main() {
  tauri::Builder::default()
      // 3. Expose commands to the frontend
      .invoke_handler(tauri::generate_handler![
          login_command,
          logout_command,
          check_auth_command
      ])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}