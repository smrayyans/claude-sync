use crate::git::{history, repo, Commit};
use crate::sync::conflict::Resolution;
use crate::sync::engine::{resolve_file_conflict, sync_repo_path};

#[tauri::command]
pub fn get_commit_history(limit: u32) -> Result<Vec<Commit>, String> {
    let sync_repo = sync_repo_path();
    if !sync_repo.exists() {
        return Ok(vec![]);
    }

    let repository = repo::open_repo(&sync_repo).map_err(|e| e.to_string())?;
    history::get_history(&repository, limit as usize).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_commit_diff(hash: String) -> Result<String, String> {
    let sync_repo = sync_repo_path();
    if !sync_repo.exists() {
        return Ok(String::new());
    }

    let repository = repo::open_repo(&sync_repo).map_err(|e| e.to_string())?;
    history::get_commit_diff(&repository, &hash).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resolve_conflict(file: String, resolution: Resolution) -> Result<(), String> {
    resolve_file_conflict(&file, resolution)
        .await
        .map_err(|e| e.to_string())
}
