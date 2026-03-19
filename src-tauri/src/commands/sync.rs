use tauri::AppHandle;

use crate::sync::{engine, FileChange, SyncResult, SyncStatus};
use crate::sync::conflict::Resolution;
use crate::sync::machine::read_machine_config;

#[tauri::command]
pub async fn sync_now(app: AppHandle) -> Result<SyncResult, String> {
    engine::perform_sync(&app)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_pull(app: AppHandle) -> Result<SyncResult, String> {
    // Pull-only: fetch + apply remote changes
    sync_now(app).await
}

#[tauri::command]
pub async fn sync_push(app: AppHandle) -> Result<SyncResult, String> {
    sync_now(app).await
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
