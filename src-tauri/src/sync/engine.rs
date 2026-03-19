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
    machine::{read_machine_config, write_machine_config, append_pull_log, read_pull_log, PullLogEntry},
    watcher::wait_for_stable,
    ChangeType, FileChange, RepoStatus, SyncResult, SyncStatus,
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
    if token.is_empty() {
        return Ok(SyncResult {
            success: false,
            files_pushed: vec![],
            files_pulled: vec![],
            conflicts: vec![],
            message: "No token found. Re-enter your PAT in Settings → Remote.".to_string(),
        });
    }

    let sync_repo = sync_repo_path();

    // Clone or open sync repo
    let repository = if sync_repo.exists() {
        repo::open_repo(&sync_repo)?
    } else {
        repo::clone_repo(&remote_url, &sync_repo, &token)?
    };

    // Fetch remote — returns false (non-fatal) when remote is empty on first push
    let remote_has_data = repo::fetch(&repository, &token)?;

    // Fast-forward sync repo to latest remote so our push is always clean
    if remote_has_data {
        let _ = repo::pull_fast_forward(&repository);
    }

    // Collect local AND remote files (remote may have new files from other machines)
    let claude_dir = claude_dir();
    let local_files = collect_tracked_files(&claude_dir);
    let remote_files = collect_remote_files(&sync_repo, &claude_dir);
    let mut all_files: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
    for (key, path) in local_files {
        all_files.insert(key, path);
    }
    for (key, path) in remote_files {
        all_files.entry(key).or_insert(path);
    }

    let mut files_pushed = vec![];
    let mut files_pulled = vec![];
    let mut conflicts = vec![];

    let mut hashes = load_hashes();

    for (file_key, local_path) in &all_files {
        // Safe sync: wait for file to stabilize
        if local_path.exists() {
            wait_for_stable(local_path).await;
        }

        // Get remote content from sync repo (only if remote had data)
        let remote_path = sync_repo.join(file_key);
        let remote_content = if remote_has_data && remote_path.exists() {
            std::fs::read(&remote_path).ok()
        } else {
            None
        };

        // On first push (empty remote), force everything local → remote
        let status = if !remote_has_data && local_path.exists() {
            ConflictStatus::LocalOnly
        } else {
            detect_conflict(local_path, remote_content.as_deref(), file_key)
        };

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

/// Scan the sync repo for files that should be pulled — covers files that
/// exist on remote but not locally (new agents from another machine, etc.).
pub fn collect_remote_files(sync_repo: &Path, claude_dir: &Path) -> Vec<(String, PathBuf)> {
    let mut files = vec![];

    for entry in WalkDir::new(sync_repo)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        // Skip .git internals
        if entry.path().to_string_lossy().contains("/.git/") || entry.path().to_string_lossy().contains("/.git") && entry.path() != sync_repo.join(".gitignore") {
            continue;
        }
        if entry.file_name().to_string_lossy() == ".gitignore" {
            continue;
        }

        if let Ok(rel) = entry.path().strip_prefix(sync_repo) {
            let key = crate::claude::normalize_path(&rel.to_string_lossy());

            // Skip excluded paths
            let local_path = claude_dir.join(&key);
            if is_excluded(&local_path, claude_dir) {
                continue;
            }

            files.push((key, local_path));
        }
    }

    files
}

pub fn get_sync_pull_log() -> Vec<PullLogEntry> {
    read_pull_log()
}

/// Pull-only: fetch remote and apply all remote files to local.
pub async fn perform_pull(_app: &AppHandle) -> Result<SyncResult> {
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
    if token.is_empty() {
        return Ok(SyncResult {
            success: false,
            files_pushed: vec![],
            files_pulled: vec![],
            conflicts: vec![],
            message: "No token found. Re-enter your PAT in Settings → Remote.".to_string(),
        });
    }

    let sync_repo = sync_repo_path();
    let repository = if sync_repo.exists() {
        repo::open_repo(&sync_repo)?
    } else {
        repo::clone_repo(&remote_url, &sync_repo, &token)?
    };

    let remote_has_data = repo::fetch(&repository, &token)?;
    if !remote_has_data {
        return Ok(SyncResult {
            success: true,
            files_pushed: vec![],
            files_pulled: vec![],
            conflicts: vec![],
            message: "Remote is empty — nothing to pull.".to_string(),
        });
    }

    repo::pull_fast_forward(&repository)?;

    let claude_dir = claude_dir();

    // Scan BOTH local files and sync repo files — the sync repo may have
    // files that don't exist locally yet (new agents from another machine).
    let local_files = collect_tracked_files(&claude_dir);
    let remote_files = collect_remote_files(&sync_repo, &claude_dir);

    // Merge into a deduplicated map (file_key → local_path)
    let mut all_files: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
    for (key, path) in local_files {
        all_files.insert(key, path);
    }
    for (key, path) in remote_files {
        all_files.entry(key).or_insert(path);
    }

    let mut files_pulled = vec![];
    let mut hashes = load_hashes();

    for (file_key, local_path) in &all_files {
        let remote_path = sync_repo.join(file_key);
        if remote_path.exists() {
            let content = std::fs::read(&remote_path)?;
            if let Some(parent) = local_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(local_path, &content)?;
            let hash = crate::sync::conflict::hash_content(&content);
            update_hash(&mut hashes, file_key, &hash);
            files_pulled.push(file_key.clone());
        }
    }

    save_hashes(&hashes)?;

    // Record pull log entry for this machine
    append_pull_log(PullLogEntry {
        machine_name: config.machine_name.clone(),
        machine_id: config.machine_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    });

    let mut config = config;
    config.last_synced = Some(chrono::Utc::now().to_rfc3339());
    write_machine_config(&config).await?;

    let n_pulled = files_pulled.len();
    Ok(SyncResult {
        success: true,
        files_pushed: vec![],
        files_pulled,
        conflicts: vec![],
        message: format!("Pulled {} files from remote", n_pulled),
    })
}

/// Push-only: commit locally changed files and push to remote.
pub async fn perform_push(_app: &AppHandle) -> Result<SyncResult> {
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
    if token.is_empty() {
        return Ok(SyncResult {
            success: false,
            files_pushed: vec![],
            files_pulled: vec![],
            conflicts: vec![],
            message: "No token found. Re-enter your PAT in Settings → Remote.".to_string(),
        });
    }

    let sync_repo = sync_repo_path();
    let repository = if sync_repo.exists() {
        repo::open_repo(&sync_repo)?
    } else {
        repo::clone_repo(&remote_url, &sync_repo, &token)?
    };

    // Non-fatal fetch (empty remote ok)
    let remote_has_data = repo::fetch(&repository, &token).unwrap_or(false);

    // Fast-forward sync repo to remote HEAD so new commit sits cleanly on top
    if remote_has_data {
        let _ = repo::pull_fast_forward(&repository);
    }

    let claude_dir = claude_dir();
    let tracked_files = collect_tracked_files(&claude_dir);
    let mut files_pushed = vec![];
    let mut files_deleted = vec![];
    let mut hashes = load_hashes();

    // Detect new/modified local files
    for (file_key, local_path) in &tracked_files {
        if !local_path.exists() {
            continue;
        }

        let committed = repo::get_file_from_head(&repository, file_key);
        let changed = match committed {
            None => true,
            Some(ref head_bytes) => {
                std::fs::read(local_path).unwrap_or_default() != *head_bytes
            }
        };

        if changed {
            let repo_path = sync_repo.join(file_key);
            if let Some(parent) = repo_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(local_path, &repo_path)?;
            if let Some(hash) = hash_file(local_path).ok() {
                update_hash(&mut hashes, file_key, &hash);
            }
            files_pushed.push(file_key.clone());
        }
    }

    // Detect files deleted locally but still in git HEAD
    if let Ok(head) = repository.head().and_then(|h| h.peel_to_tree()) {
        head.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
            if entry.kind() != Some(git2::ObjectType::Blob) {
                return git2::TreeWalkResult::Ok;
            }
            let file_key = if dir.is_empty() {
                entry.name().unwrap_or("").to_string()
            } else {
                format!("{}{}", dir, entry.name().unwrap_or(""))
            };
            // Skip .gitignore
            if file_key == ".gitignore" {
                return git2::TreeWalkResult::Ok;
            }
            let local_path = claude_dir.join(&file_key);
            if !local_path.exists() {
                // File in HEAD but deleted locally — remove from sync repo
                let repo_path = sync_repo.join(&file_key);
                let _ = std::fs::remove_file(&repo_path);
                hashes.hashes.remove(&file_key);
                files_deleted.push(file_key);
            }
            git2::TreeWalkResult::Ok
        }).ok();
    }

    save_hashes(&hashes)?;

    let all_changes: Vec<String> = files_pushed.iter().chain(files_deleted.iter()).cloned().collect();

    if !all_changes.is_empty() {
        let changed_list = files_pushed.iter().map(|f| format!("  + {f}"))
            .chain(files_deleted.iter().map(|f| format!("  - {f} (deleted)")))
            .collect::<Vec<_>>()
            .join("\n");
        let message = format!(
            "[{}] push: {} changed, {} deleted\n\nChanges:\n{}",
            config.machine_name,
            files_pushed.len(),
            files_deleted.len(),
            changed_list
        );
        repo::stage_and_commit(&repository, &all_changes, &message)?;
        repo::push(&repository, &token)?;

        let mut config = config;
        config.last_synced = Some(chrono::Utc::now().to_rfc3339());
        write_machine_config(&config).await?;

        let msg = match (files_pushed.len(), files_deleted.len()) {
            (p, 0) => format!("Pushed {p} files to remote"),
            (0, d) => format!("Deleted {d} files from remote"),
            (p, d) => format!("Pushed {p} files, deleted {d} from remote"),
        };
        return Ok(SyncResult {
            success: true,
            files_pushed: all_changes,
            files_pulled: vec![],
            conflicts: vec![],
            message: msg,
        });
    }

    // No new file changes — but maybe a previous push committed locally and
    // failed before reaching GitHub. Push any local commits that are ahead.
    let ahead = repo::count_ahead(&repository).unwrap_or(0);
    if ahead > 0 {
        repo::push(&repository, &token)?;

        let mut config = config;
        config.last_synced = Some(chrono::Utc::now().to_rfc3339());
        write_machine_config(&config).await?;

        return Ok(SyncResult {
            success: true,
            files_pushed: vec![],
            files_pulled: vec![],
            conflicts: vec![],
            message: format!("Pushed {ahead} pending commit(s) to remote"),
        });
    }

    Ok(SyncResult {
        success: true,
        files_pushed: vec![],
        files_pulled: vec![],
        conflicts: vec![],
        message: "Nothing to push — already up to date.".to_string(),
    })
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

/// Non-destructive status check: fetch remote + compare local vs sync repo.
/// Does NOT modify any files or hashes.
pub async fn check_repo_status() -> RepoStatus {
    let config = match read_machine_config().await {
        Ok(c) => c,
        Err(e) => {
            return RepoStatus {
                local_changes: vec![],
                commits_behind: 0,
                is_online: false,
                error: Some(e.to_string()),
            };
        }
    };

    let remote_url = match &config.remote_url {
        Some(url) => url.clone(),
        None => {
            return RepoStatus {
                local_changes: vec![],
                commits_behind: 0,
                is_online: false,
                error: Some("No remote configured".to_string()),
            };
        }
    };

    let is_online = check_online().await;

    let token = auth::get_token(&remote_url).unwrap_or_default();
    let sync_repo = sync_repo_path();

    // Open or clone sync repo
    let repository = match if sync_repo.exists() {
        repo::open_repo(&sync_repo)
    } else {
        repo::clone_repo(&remote_url, &sync_repo, &token)
    } {
        Ok(r) => r,
        Err(e) => {
            return RepoStatus {
                local_changes: vec![],
                commits_behind: 0,
                is_online,
                error: Some(format!("Cannot open sync repo: {e}")),
            };
        }
    };

    // Fetch from remote to update origin/main
    let mut commits_behind = 0usize;
    if is_online && !token.is_empty() {
        if let Ok(true) = repo::fetch(&repository, &token) {
            // Count how many commits remote is ahead of local
            if let (Ok(local_head), Ok(remote_ref)) = (
                repository.head().and_then(|h| h.peel_to_commit()),
                repository.find_reference("refs/remotes/origin/main")
                    .and_then(|r| r.peel_to_commit()),
            ) {
                if let Ok((_, behind)) = repository.graph_ahead_behind(local_head.id(), remote_ref.id()) {
                    commits_behind = behind;
                }
            }
        }
    }

    // Compare local files vs what's in the sync repo (ground truth of last push)
    let claude_dir = claude_dir();
    let tracked = collect_tracked_files(&claude_dir);
    let mut local_changes = vec![];

    for (file_key, local_path) in &tracked {
        let repo_path = sync_repo.join(file_key);

        match (local_path.exists(), repo_path.exists()) {
            (true, false) => {
                // Local file exists but not in sync repo → added locally
                let size_bytes = local_path.metadata().map(|m| m.len()).ok();
                local_changes.push(FileChange {
                    path: file_key.clone(),
                    change_type: ChangeType::Added,
                    size_bytes,
                });
            }
            (false, true) => {
                // In sync repo but deleted locally
                local_changes.push(FileChange {
                    path: file_key.clone(),
                    change_type: ChangeType::Deleted,
                    size_bytes: None,
                });
            }
            (true, true) => {
                // Both exist — compare content hashes
                let local_hash = hash_file(local_path).unwrap_or_default();
                let repo_hash = hash_file(&repo_path).unwrap_or_default();
                if local_hash != repo_hash {
                    let size_bytes = local_path.metadata().map(|m| m.len()).ok();
                    local_changes.push(FileChange {
                        path: file_key.clone(),
                        change_type: ChangeType::Modified,
                        size_bytes,
                    });
                }
            }
            (false, false) => {}
        }
    }

    RepoStatus {
        local_changes,
        commits_behind,
        is_online,
        error: None,
    }
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
