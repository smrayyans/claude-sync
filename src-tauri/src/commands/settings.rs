use serde::{Deserialize, Serialize};

use crate::claude::settings::{read_settings, write_settings, ClaudeSettings};
use crate::git::{auth, repo};
use crate::sync::machine::{read_machine_config, write_machine_config, MachineConfig};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASES_API: &str = "https://api.github.com/repos/smrayyans/claude-sync/releases/latest";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_url: String,
    pub release_notes: String,
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let result = tokio::task::spawn_blocking(|| {
        reqwest::blocking::Client::builder()
            .user_agent("claude-sync-updater")
            .build()
            .map_err(|e| e.to_string())?
            .get(RELEASES_API)
            .send()
            .map_err(|e| e.to_string())?
            .json::<serde_json::Value>()
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    let latest_version = result["tag_name"]
        .as_str()
        .unwrap_or("unknown")
        .trim_start_matches('v')
        .to_string();

    let release_url = result["html_url"]
        .as_str()
        .unwrap_or(RELEASES_API)
        .to_string();

    let release_notes = result["body"]
        .as_str()
        .unwrap_or("")
        .chars()
        .take(300)
        .collect::<String>();

    let update_available = is_newer(&latest_version, CURRENT_VERSION);

    Ok(UpdateInfo {
        current_version: CURRENT_VERSION.to_string(),
        latest_version,
        update_available,
        release_url,
        release_notes,
    })
}

/// Returns true if `a` is a higher semver than `b`.
fn is_newer(a: &str, b: &str) -> bool {
    let parse = |s: &str| -> (u32, u32, u32) {
        let mut parts = s.splitn(3, '.');
        let major = parts.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        let minor = parts.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        let patch = parts.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    };
    parse(a) > parse(b)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub claude: ClaudeSettings,
    pub machine: MachineConfig,
}

#[tauri::command]
pub async fn get_app_settings() -> Result<AppSettings, String> {
    let claude = read_settings().map_err(|e| e.to_string())?;
    let machine = read_machine_config().await.map_err(|e| e.to_string())?;
    Ok(AppSettings { claude, machine })
}

#[tauri::command]
pub async fn save_app_settings(settings: AppSettings) -> Result<(), String> {
    write_settings(&settings.claude).map_err(|e| e.to_string())?;
    write_machine_config(&settings.machine)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_machine_config() -> Result<MachineConfig, String> {
    read_machine_config().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_machine_config(config: MachineConfig) -> Result<(), String> {
    write_machine_config(&config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn setup_remote(url: String, token: String) -> Result<(), String> {
    // Store token in keychain
    auth::store_token(&url, &token).map_err(|e| e.to_string())?;

    // Update machine config with remote URL
    let mut config = read_machine_config().await.map_err(|e| e.to_string())?;
    config.remote_url = Some(url.clone());
    write_machine_config(&config)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn test_remote_connection() -> bool {
    let config = match read_machine_config().await {
        Ok(c) => c,
        Err(_) => return false,
    };

    let remote_url = match &config.remote_url {
        Some(url) => url.clone(),
        None => return false,
    };

    let token = auth::get_token(&remote_url).unwrap_or_default();
    repo::test_connection(&remote_url, &token)
}
