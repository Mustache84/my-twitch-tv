use anyhow::{anyhow, Result};
use keyring::Entry;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenResponse,
    TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::oneshot;
use warp::Filter;

// --- CONFIGURATION ---
// specific service name for the OS keyring
const SERVICE_NAME: &str = "my-twitch-tv-app";
const USER_ACCOUNT: &str = "twitch_access_token";

// You will need to register an app at https://dev.twitch.tv/console
// For now, you can use a placeholder, but the login won't fully complete without a real ID.
const CLIENT_ID: &str = "YOUR_TWITCH_CLIENT_ID_HERE"; 
const REDIRECT_URL: &str = "http://localhost:3000";

#[derive(Serialize, Clone)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub username: Option<String>,
}

pub async fn login_flow() -> Result<String> {
    // 1. Setup OAuth Client
    let client = BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        None, // No client secret needed for PKCE flow!
        AuthUrl::new("https://id.twitch.tv/oauth2/authorize".to_string())?,
        Some(TokenUrl::new("https://id.twitch.tv/oauth2/token".to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(REDIRECT_URL.to_string())?);

    // 2. Generate PKCE Challenge (Security Best Practice)
    // This prevents code injection attacks.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // 3. Generate the Authorization URL
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:read:follows".to_string())) // See live channels
        .add_scope(Scope::new("chat:read".to_string()))         // Read chat
        .add_scope(Scope::new("chat:edit".to_string()))         // Send chat
        .set_pkce_challenge(pkce_challenge)
        .url();

    // 4. Spin up a temporary local server to capture the redirect
    let (tx, rx) = oneshot::channel::<String>();
    let tx = Arc::new(tokio::sync::Mutex::new(Some(tx)));

    let route = warp::get()
        .and(warp::query::<Vec<(String, String)>>())
        .map(move |params: Vec<(String, String)>| {
            let code = params
                .iter()
                .find(|(k, _)| k == "code")
                .map(|(_, v)| v.clone());

            if let Some(c) = code {
                if let Some(tx) = tx.clone().try_lock().ok().and_then(|mut guard| guard.take()) {
                    let _ = tx.send(c);
                }
                return "Login successful! You can close this tab and return to the app.";
            }
            "Error: Missing code param."
        });

    // Spawn the server in the background
    let server = warp::serve(route).bind(([127, 0, 0, 1], 3000));
    let server_handle = tokio::spawn(server);

    // 5. Open the User's Browser
    println!("Opening browser to: {}", auth_url);
    // In a real Tauri app, we use the `shell` API to open this, 
    // but for the backend logic we can just return the URL to the frontend 
    // or open it here if we have system access. 
    // For now, we assume the frontend calls 'open' on this URL.
    open::that(auth_url.to_string())?;

    // 6. Wait for the redirect code
    let code = rx.await?;
    
    // Stop the server (the handle will be dropped, but actual shutdown depends on runtime)
    server_handle.abort(); 

    // 7. Exchange the Authorization Code for an Access Token
    let token_result = client
        .exchange_code(oauth2::AuthorizationCode::new(code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .map_err(|e| anyhow!("Token exchange failed: {}", e))?;

    let access_token = token_result.access_token().secret();

    // 8. Save securely to OS Keyring
    save_token_securely(access_token)?;

    Ok("Authentication Successful".to_string())
}

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
        Err(keyring::Error::NoEntry) => Ok(()), // Already logged out
        Err(e) => Err(anyhow!(e)),
    }
}
