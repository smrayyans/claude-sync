use tauri::AppHandle;

use crate::sync::{engine, FileChange, RepoStatus, SyncResult, SyncStatus};
use crate::sync::machine::{read_machine_config, PullLogEntry};

#[tauri::command]
pub async fn sync_now(app: AppHandle) -> Result<SyncResult, String> {
    engine::perform_sync(&app)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_pull(app: AppHandle) -> Result<SyncResult, String> {
    engine::perform_pull(&app)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_push(app: AppHandle) -> Result<SyncResult, String> {
    engine::perform_push(&app)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_pull_log() -> Vec<PullLogEntry> {
    engine::get_sync_pull_log()
}

#[tauri::command]
pub async fn get_sync_status() -> SyncStatus {
    let config = read_machine_config().await.unwrap_or_default();
    let is_online = engine::check_online().await;
    let pending = engine::get_pending_changes();

    SyncStatus {
        pending_changes: pending.len(),
        last_synced: config.last_synced,
        machine_name: config.machine_name,
        is_online,
        is_syncing: false,
        error: None,
    }
}

#[tauri::command]
pub async fn get_pending_changes() -> Vec<FileChange> {
    engine::get_pending_changes()
}

#[tauri::command]
pub async fn check_repo_status() -> RepoStatus {
    engine::check_repo_status().await
}
