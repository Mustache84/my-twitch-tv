const { invoke } = window.__TAURI__.core;

// --- STATE MANAGEMENT (This is the Auto-Login Trigger) ---
async function checkAuthAndLoad() {
    try {
        console.log("Checking auth state...");
        const isAuth = await invoke('check_auth_command');
        if (isAuth) {
            console.log("Auth valid. Loading dashboard.");
            showDashboard();
            loadLiveChannels();
        } else {
            console.log("Auth missing. Showing login.");
            showLogin();
        }
    } catch (err) {
        console.error("Auth Check Error:", err);
    }
}

// --- API CALLS ---
async function loadLiveChannels() {
    const grid = document.getElementById('grid');
    grid.innerHTML = '<div style="color:#777;">Loading streams...</div>';

    try {
        const streams = await invoke('get_live_channels_command');
        grid.innerHTML = ''; 

        if (streams.length === 0) {
            grid.innerHTML = '<div style="padding:20px;">No followed channels are live.</div>';
            return;
        }

        streams.forEach(stream => {
            const thumbUrl = stream.thumbnail_url
                .replace('{width}', '440')
                .replace('{height}', '248');

            const card = document.createElement('div');
            card.className = 'card';
            // Placeholder for Video Player
            card.onclick = () => alert("Play stream: " + stream.user_login); 

            card.innerHTML = `
                <img src="${thumbUrl}" class="thumb" loading="lazy" />
                <div class="info">
                    <div class="title" title="${stream.title}">${stream.title}</div>
                    <div class="meta">
                        <span>${stream.user_name}</span>
                        <span class="live-tag">● ${formatViewers(stream.viewer_count)}</span>
                    </div>
                    <div class="meta" style="margin-top:4px; font-size:0.75rem;">
                        ${stream.game_name}
                    </div>
                </div>
            `;
            grid.appendChild(card);
        });

    } catch (err) {
        grid.innerHTML = `<div style="color:red">Error loading streams: ${err}</div>`;
    }
}

function formatViewers(count) {
    if (count >= 1000) return (count / 1000).toFixed(1) + 'k';
    return count;
}

// --- UI SWITCHING ---
const elLogin = document.getElementById('login-screen');
const elDash = document.getElementById('app-dashboard');

function showLogin() { 
    if(elLogin) elLogin.style.display = 'flex'; 
    if(elDash) elDash.style.display = 'none'; 
}
function showDashboard() { 
    if(elLogin) elLogin.style.display = 'none'; 
    if(elDash) elDash.style.display = 'block'; 
}

// --- EVENT LISTENERS ---
document.addEventListener('DOMContentLoaded', () => {
    // 1. TRIGGER AUTO-LOGIN ON START
    checkAuthAndLoad();

    // 2. Setup Login Button
    const btnLogin = document.getElementById('btn-login');
    if (btnLogin) {
        btnLogin.addEventListener('click', async () => {
            const status = document.getElementById('login-status');
            try {
                status.innerText = "Getting Code...";
                const authData = await invoke('start_login_command');
                
                status.innerHTML = `
                    Go to <strong>twitch.tv/activate</strong><br>
                    Code: <span style="font-size:1.5em; color:#9146FF">${authData.user_code}</span>
                `;
                window.open(authData.verification_uri, '_blank');

                await invoke('finish_login_command', { 
                    deviceCode: authData.device_code, 
                    interval: parseInt(authData.interval)
                });
                
                showDashboard();
                loadLiveChannels();

            } catch (e) {
                status.innerText = "Error: " + e;
            }
        });
    }

    // 3. Setup Logout Button
    const btnLogout = document.getElementById('btn-logout');
    if (btnLogout) {
        btnLogout.addEventListener('click', async () => {
            await invoke('logout_command');
            showLogin();
            document.getElementById('login-status').innerText = "Logged out.";
        });
    }
});