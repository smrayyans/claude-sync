use tauri::{AppHandle, Manager};
use serde::{Deserialize, Serialize};

use crate::SyncLock;
use crate::sync::{engine, FileChange, RepoStatus, SyncResult, SyncStatus};
use crate::sync::machine::{read_machine_config, PullLogEntry};
use crate::git::{auth, repo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushDiagnostic {
    pub remote_url: Option<String>,
    pub token_found: bool,
    pub sync_repo_exists: bool,
    pub sync_repo_path: String,
    pub remote_has_data: bool,
    pub head_commit: Option<String>,
    pub commits_ahead: usize,
    pub tracked_files_count: usize,
    pub files_to_push: Vec<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn sync_now(app: AppHandle) -> Result<SyncResult, String> {
    let lock = app.state::<SyncLock>();
    let _guard = lock.0.lock().await;
    engine::perform_sync(&app)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_pull(app: AppHandle) -> Result<SyncResult, String> {
    let lock = app.state::<SyncLock>();
    let _guard = lock.0.lock().await;
    engine::perform_pull(&app)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_push(app: AppHandle) -> Result<SyncResult, String> {
    let lock = app.state::<SyncLock>();
    let _guard = lock.0.lock().await;
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

#[tauri::command]
pub async fn diagnose_push() -> PushDiagnostic {
    use crate::sync::conflict::hash_file;
    use crate::sync::engine::{sync_repo_path, collect_tracked_files};
    use crate::claude::claude_dir;

    let config = match read_machine_config().await {
        Ok(c) => c,
        Err(e) => return PushDiagnostic {
            remote_url: None, token_found: false, sync_repo_exists: false,
            sync_repo_path: String::new(), remote_has_data: false,
            head_commit: None, commits_ahead: 0, tracked_files_count: 0,
            files_to_push: vec![], error: Some(format!("read_machine_config: {e}")),
        },
    };

    let remote_url = config.remote_url.clone();
    let token = remote_url.as_ref()
        .and_then(|u| auth::get_token(u).ok())
        .unwrap_or_default();
    let token_found = !token.is_empty();

    let sync_repo = sync_repo_path();
    let sync_repo_exists = sync_repo.exists();
    let sync_repo_path_str = sync_repo.display().to_string();

    let repository = match remote_url.as_ref() {
        None => return PushDiagnostic {
            remote_url, token_found, sync_repo_exists,
            sync_repo_path: sync_repo_path_str, remote_has_data: false,
            head_commit: None, commits_ahead: 0, tracked_files_count: 0,
            files_to_push: vec![], error: Some("No remote URL configured".into()),
        },
        Some(url) => {
            if sync_repo_exists {
                match repo::open_repo(&sync_repo) {
                    Ok(r) => r,
                    Err(e) => return PushDiagnostic {
                        remote_url: Some(url.clone()), token_found, sync_repo_exists,
                        sync_repo_path: sync_repo_path_str, remote_has_data: false,
                        head_commit: None, commits_ahead: 0, tracked_files_count: 0,
                        files_to_push: vec![], error: Some(format!("open_repo: {e}")),
                    },
                }
            } else {
                match repo::clone_repo(url, &sync_repo, &token) {
                    Ok(r) => r,
                    Err(e) => return PushDiagnostic {
                        remote_url: Some(url.clone()), token_found, sync_repo_exists,
                        sync_repo_path: sync_repo_path_str, remote_has_data: false,
                        head_commit: None, commits_ahead: 0, tracked_files_count: 0,
                        files_to_push: vec![], error: Some(format!("clone_repo: {e}")),
                    },
                }
            }
        }
    };

    let remote_has_data = repo::fetch(&repository, &token).unwrap_or(false);
    if remote_has_data {
        let _ = repo::pull_fast_forward(&repository);
    }

    let head_commit = repository.head().ok()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|c| format!("{} — {}", &c.id().to_string()[..8], c.summary().unwrap_or("(no msg)")));

    let commits_ahead = repo::count_ahead(&repository).unwrap_or(0);

    let claude_dir = claude_dir();
    let tracked_files = collect_tracked_files(&claude_dir, &[]);
    let tracked_files_count = tracked_files.len();
    let mut files_to_push = vec![];

    for (file_key, local_path) in &tracked_files {
        if !local_path.exists() { continue; }
        let committed = repo::get_file_from_head(&repository, file_key);
        let changed = match committed {
            None => true,
            Some(ref head_bytes) => std::fs::read(local_path).unwrap_or_default() != *head_bytes,
        };
        if changed {
            files_to_push.push(format!("{} (local: {}b)",
                file_key,
                local_path.metadata().map(|m| m.len()).unwrap_or(0)
            ));
        }
    }

    PushDiagnostic {
        remote_url, token_found, sync_repo_exists,
        sync_repo_path: sync_repo_path_str, remote_has_data,
        head_commit, commits_ahead, tracked_files_count,
        files_to_push, error: None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreview {
    pub file_key: String,
    pub local_path: String,
    pub local_content: Option<String>,
    pub sync_content: Option<String>,
}

/// Return the current local content and last committed content for a file key.
/// Used by the UI to show a preview when clicking a pending change.
#[tauri::command]
pub fn get_file_preview(file_key: String) -> Result<FilePreview, String> {
    let (local_content, sync_content, local_path) =
        engine::get_file_preview_data(&file_key);
    Ok(FilePreview { file_key, local_path, local_content, sync_content })
}

/// Push only the explicitly selected file keys (selective push from the UI dialog).
#[tauri::command]
pub async fn sync_push_selective(app: AppHandle, file_keys: Vec<String>) -> Result<SyncResult, String> {
    let lock = app.state::<SyncLock>();
    let _guard = lock.0.lock().await;
    let selected: std::collections::HashSet<String> = file_keys.into_iter().collect();
    engine::perform_push_selective(&app, selected)
        .await
        .map_err(|e| e.to_string())
}
