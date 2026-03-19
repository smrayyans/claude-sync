use serde::{Deserialize, Serialize};

use crate::claude::settings::{read_settings, write_settings, ClaudeSettings};
use crate::git::{auth, repo};
use crate::sync::machine::{read_machine_config, write_machine_config, MachineConfig};

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
