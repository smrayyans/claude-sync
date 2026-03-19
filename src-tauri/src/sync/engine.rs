use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use walkdir::WalkDir;

use crate::claude::{agents_dir, claude_dir, projects_dir, skills_dir};
use crate::git::{auth, repo};
use crate::sync::{
    conflict::{
        detect_conflict, load_hashes, save_hashes, update_hash, hash_file,
        apply_resolution, ConflictStatus, Resolution,
    },
    machine::{read_machine_config, write_machine_config},
    watcher::wait_for_stable,
    ChangeType, FileChange, SyncResult, SyncStatus,
};

// Files/dirs to never sync
const EXCLUDED_PATHS: &[&str] = &[
    ".credentials.json",
    "sessions",
    "file-history",
    "cache",
    "history.jsonl",
    "backups",
    "telemetry",
];

fn is_excluded(path: &Path, claude_dir: &Path) -> bool {
    if let Ok(rel) = path.strip_prefix(claude_dir) {
        let rel_str = rel.to_string_lossy();
        for excluded in EXCLUDED_PATHS {
            if rel_str.starts_with(excluded) || rel_str.contains(excluded) {
                return true;
            }
        }
    }
    false
}

pub fn sync_repo_path() -> PathBuf {
    dirs::home_dir()
        .expect("home dir")
        .join(".claude-sync")
        .join("sync-repo")
}

pub async fn start_auto_sync(app: AppHandle) {
    let config = read_machine_config().await.unwrap_or_default();
    let interval = Duration::from_secs(config.auto_sync_interval * 60);

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;

            // Connectivity check
            if !check_online().await {
                let _ = app.emit("sync-status", SyncStatus {
                    pending_changes: 0,
                    last_synced: None,
                    machine_name: config.machine_name.clone(),
                    is_online: false,
                    is_syncing: false,
                    error: Some("Offline — changes queued".to_string()),
                });
                continue;
            }

            let _ = app.emit("sync-started", ());
            match perform_sync(&app).await {
                Ok(result) => {
                    let _ = app.emit("sync-completed", result);
                }
                Err(e) => {
                    log::error!("Auto-sync failed: {e}");
                    let _ = app.emit("sync-error", e.to_string());
                }
            }
        }
    });
}

pub async fn check_online() -> bool {
    tokio::task::spawn_blocking(|| {
        reqwest::blocking::Client::new()
            .head("https://api.github.com")
            .timeout(Duration::from_secs(5))
            .send()
            .is_ok()
    })
    .await
    .unwrap_or(false)
}

pub async fn perform_sync(app: &AppHandle) -> Result<SyncResult> {
    let config = read_machine_config().await?;

    let remote_url = match &config.remote_url {
        Some(url) => url.clone(),
        None => {
            return Ok(SyncResult {
                success: false,
                files_pushed: vec![],
                files_pulled: vec![],
                conflicts: vec![],
                message: "No remote configured. Please set up a remote in Settings.".to_string(),
            });
        }
    };

    let token = auth::get_token(&remote_url).unwrap_or_default();
    let sync_repo = sync_repo_path();

    // Clone or open sync repo
    let repository = if sync_repo.exists() {
        repo::open_repo(&sync_repo)?
    } else {
        repo::clone_repo(&remote_url, &sync_repo, &token)?
    };

    // Fetch remote
    repo::fetch(&repository, &token)?;

    // Collect local files to sync
    let claude_dir = claude_dir();
    let tracked_files = collect_tracked_files(&claude_dir);

    let mut files_pushed = vec![];
    let mut files_pulled = vec![];
    let mut conflicts = vec![];

    let mut hashes = load_hashes();

    for (file_key, local_path) in &tracked_files {
        // Safe sync: wait for file to stabilize
        if local_path.exists() {
            wait_for_stable(local_path).await;
        }

        // Get remote content from sync repo
        let remote_path = sync_repo.join(file_key);
        let remote_content = if remote_path.exists() {
            std::fs::read(&remote_path).ok()
        } else {
            None
        };

        let status = detect_conflict(
            local_path,
            remote_content.as_deref(),
            file_key,
        );

        match status {
            ConflictStatus::Unchanged => {}
            ConflictStatus::LocalOnly => {
                // Copy local → sync repo
                if local_path.exists() {
                    if let Some(parent) = remote_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::copy(local_path, &remote_path)?;
                    let hash = hash_file(local_path).unwrap_or_default();
                    update_hash(&mut hashes, file_key, &hash);
                    files_pushed.push(file_key.clone());
                }
            }
            ConflictStatus::RemoteOnly => {
                // Copy sync repo → local
                if let Some(content) = &remote_content {
                    if let Some(parent) = local_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(local_path, content)?;
                    let hash = crate::sync::conflict::hash_content(content);
                    update_hash(&mut hashes, file_key, &hash);
                    files_pulled.push(file_key.clone());
                }
            }
            ConflictStatus::Conflict => {
                conflicts.push(file_key.clone());
                let _ = app.emit("sync-conflict", file_key.clone());
            }
        }
    }

    save_hashes(&hashes)?;

    // Commit and push if anything was pushed
    if !files_pushed.is_empty() {
        let changed_list = files_pushed
            .iter()
            .map(|f| format!("  - {f} (modified)"))
            .collect::<Vec<_>>()
            .join("\n");

        let message = format!(
            "[{}] sync: agents({}) skills({}) memory({}) settings\n\nChanged:\n{}",
            config.machine_name,
            files_pushed.iter().filter(|f| f.starts_with("agents/")).count(),
            files_pushed.iter().filter(|f| f.starts_with("skills/")).count(),
            files_pushed.iter().filter(|f| f.starts_with("memory/")).count(),
            changed_list
        );

        repo::stage_and_commit(&repository, &files_pushed, &message)?;
        repo::push(&repository, &token)?;
    }

    // Update last synced time
    let mut config = config;
    config.last_synced = Some(chrono::Utc::now().to_rfc3339());
    write_machine_config(&config).await?;

    let success = conflicts.is_empty();
    let message = if conflicts.is_empty() {
        format!(
            "Synced: {} pushed, {} pulled",
            files_pushed.len(),
            files_pulled.len()
        )
    } else {
        format!(
            "{} conflicts detected — resolve them in the Conflicts view",
            conflicts.len()
        )
    };

    Ok(SyncResult {
        success,
        files_pushed,
        files_pulled,
        conflicts,
        message,
    })
}

pub fn collect_tracked_files(claude_dir: &Path) -> Vec<(String, PathBuf)> {
    let mut files = vec![];

    // settings.json
    let settings = claude_dir.join("settings.json");
    files.push(("settings.json".to_string(), settings));

    // agents/
    let agents_dir = agents_dir();
    if agents_dir.exists() {
        for entry in WalkDir::new(&agents_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if is_excluded(entry.path(), claude_dir) {
                continue;
            }
            if let Ok(rel) = entry.path().strip_prefix(claude_dir) {
                let key = crate::claude::normalize_path(&rel.to_string_lossy());
                files.push((key, entry.path().to_path_buf()));
            }
        }
    }

    // skills/  (user-invocable slash commands)
    let skills_dir = skills_dir();
    if skills_dir.exists() {
        for entry in WalkDir::new(&skills_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if is_excluded(entry.path(), claude_dir) {
                continue;
            }
            if let Ok(rel) = entry.path().strip_prefix(claude_dir) {
                let key = crate::claude::normalize_path(&rel.to_string_lossy());
                files.push((key, entry.path().to_path_buf()));
            }
        }
    }

    // projects/*/*.jsonl  (chat sessions — sync the JSONL files)
    let projects_dir_for_history = projects_dir();
    if projects_dir_for_history.exists() {
        for entry in WalkDir::new(&projects_dir_for_history)
            .max_depth(2)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file()
                    && e.path().extension().map_or(false, |ext| ext == "jsonl")
                    && !e.path().to_string_lossy().contains("/subagents/")
            })
        {
            if is_excluded(entry.path(), claude_dir) {
                continue;
            }
            if let Ok(rel) = entry.path().strip_prefix(claude_dir) {
                let key = crate::claude::normalize_path(&rel.to_string_lossy());
                files.push((key, entry.path().to_path_buf()));
            }
        }
    }

    // projects/*/memory/
    let projects_dir = projects_dir();
    if projects_dir.exists() {
        for entry in WalkDir::new(&projects_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if is_excluded(entry.path(), claude_dir) {
                continue;
            }

            // Only include files inside a `memory` subdirectory
            let in_memory = entry.path()
                .ancestors()
                .any(|a| a.file_name().map_or(false, |n| n == "memory"));

            if !in_memory {
                continue;
            }

            if let Ok(rel) = entry.path().strip_prefix(claude_dir) {
                let key = crate::claude::normalize_path(&rel.to_string_lossy());
                files.push((key, entry.path().to_path_buf()));
            }
        }
    }

    files
}

pub fn get_pending_changes() -> Vec<FileChange> {
    let claude_dir = claude_dir();
    let tracked = collect_tracked_files(&claude_dir);
    let hashes = load_hashes();
    let mut changes = vec![];

    for (file_key, local_path) in &tracked {
        let stored = hashes.hashes.get(file_key);
        let current = if local_path.exists() {
            hash_file(local_path).ok()
        } else {
            None
        };

        let change = match (stored, &current) {
            (None, Some(_)) => Some(ChangeType::Added),
            (Some(_), None) => Some(ChangeType::Deleted),
            (Some(s), Some(c)) if s != c => Some(ChangeType::Modified),
            _ => None,
        };

        if let Some(ct) = change {
            let size_bytes = local_path.metadata().map(|m| m.len()).ok();
            changes.push(FileChange {
                path: file_key.clone(),
                change_type: ct,
                size_bytes,
            });
        }
    }

    changes
}

pub async fn resolve_file_conflict(
    file_key: &str,
    resolution: Resolution,
) -> Result<()> {
    let claude_dir = claude_dir();
    let local_path = crate::claude::native_path(file_key);
    let local_full = claude_dir.join(local_path);

    let sync_repo = sync_repo_path();
    let remote_path = sync_repo.join(file_key);
    let remote_content = std::fs::read(&remote_path).unwrap_or_default();

    let resolved = apply_resolution(&local_full, &remote_content, &resolution)?;

    // Update hashes
    let mut hashes = load_hashes();
    let hash = crate::sync::conflict::hash_content(resolved.as_bytes());
    update_hash(&mut hashes, file_key, &hash);
    save_hashes(&hashes)?;

    Ok(())
}
