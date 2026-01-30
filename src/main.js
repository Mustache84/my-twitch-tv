document.addEventListener('DOMContentLoaded', async () => {
  const statusEl = document.getElementById('status');
  
  if (!window.__TAURI__) {
    statusEl.innerText = "Error: Tauri not found.";
    return;
  }

  const { invoke } = window.__TAURI__.core;

  // --- LOGIN BUTTON LOGIC ---
  const btnLogin = document.getElementById('btn-login');
  if (btnLogin) {
    btnLogin.addEventListener('click', async () => {
      try {
        statusEl.innerText = "Connecting to Twitch...";
        
        // STEP 1: Call 'start_login_command' (Matches Rust)
        console.log("Calling start_login_command...");
        const authData = await invoke('start_login_command');
        
        // UI UPDATE: Show the code
        statusEl.innerHTML = `
          <div style="background:#333; padding:15px; border-radius:5px; margin-top:10px;">
            <p>Please go to <strong>twitch.tv/activate</strong></p>
            <p>Enter this code:</p>
            <h2 style="color:#9146FF; font-size: 2em; letter-spacing: 5px;">${authData.user_code}</h2>
            <p><small>Waiting for you to authorize...</small></p>
          </div>
        `;

        // OPTIONAL: Try to open the link automatically
        // Note: 'open' might require permission in tauri.conf.json or use window.open
        window.open(authData.verification_uri, '_blank'); 

        // STEP 2: Call 'finish_login_command' (Matches Rust)
        console.log("Polling for token...");
        const result = await invoke('finish_login_command', { 
          deviceCode: authData.device_code, 
          interval: parseInt(authData.interval) // Ensure it sends a number
        });

        statusEl.innerHTML = `<h2 style="color:#0f0">${result}</h2>`;
        
      } catch (e) {
        console.error(e);
        statusEl.innerText = "Login Error: " + e;
      }
    });
  }
});