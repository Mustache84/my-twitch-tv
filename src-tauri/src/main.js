const { invoke } = window.__TAURI__.tauri;

const elLoginScreen = document.getElementById('login-screen');
const elAppContainer = document.getElementById('app-container');
const elStatus = document.getElementById('status-log');

// --- Helper Functions ---
function showApp() {
  elLoginScreen.style.display = 'none';
  elAppContainer.style.display = 'block';
}

function showLogin() {
  elLoginScreen.style.display = 'flex';
  elAppContainer.style.display = 'none';
}

function log(msg) {
  elStatus.innerText = msg;
  console.log(msg);
}

// --- Button Listeners ---

document.getElementById('btn-login').addEventListener('click', async () => {
  try {
    log("Opening secure browser...");
    // 1. Call Rust "login_command"
    const result = await invoke('login_command');
    log(result);
    // If successful, switch view
    showApp();
  } catch (err) {
    log("Login Failed: " + err);
  }
});

document.getElementById('btn-logout').addEventListener('click', async () => {
  try {
    await invoke('logout_command');
    showLogin();
    log("Logged out securely.");
  } catch (err) {
    log("Logout error: " + err);
  }
});

// --- On Startup ---
async function init() {
  // Check if we already have a token
  const isAuthenticated = await invoke('check_auth_command');
  if (isAuthenticated) {
    showApp();
  } else {
    showLogin();
  }
}

init();