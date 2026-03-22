use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use super::projects_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub project_slug: String,
    pub project_display: String,
    pub timestamp: String,
    pub message_count: usize,
    pub first_user_message: String,
    pub path: String,
    pub file_size_bytes: u64,
    pub line_count: usize,
    pub is_synced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,     // "user" | "assistant"
    pub content: String,
    pub timestamp: String,
    pub is_tool_use: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDetail {
    pub session: ChatSession,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawEntry {
    #[serde(rename = "type")]
    entry_type: Option<String>,
    message: Option<RawMessage>,
    timestamp: Option<String>,
    #[serde(rename = "isMeta")]
    is_meta: Option<bool>,
    #[serde(rename = "isSidechain")]
    is_sidechain: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawMessage {
    role: Option<String>,
    content: Option<serde_json::Value>,
}

fn extract_text(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|item| {
                let t = item.get("type")?.as_str()?;
                match t {
                    "text" => item.get("text")?.as_str().map(|s| s.to_string()),
                    "tool_use" => {
                        let name = item.get("name")?.as_str()?;
                        Some(format!("[Tool: {name}]"))
                    }
                    "tool_result" => None, // skip tool results
                    _ => None,
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn is_noise(content: &str) -> bool {
    content.is_empty()
        || content.starts_with("<local-command")
        || content.starts_with("<command-name>")
        || content.starts_with("<system-reminder")
        || content.starts_with("<task-notification")
        || content.trim() == "[Tool: TodoWrite]"
        || content.trim() == "[Tool: TodoRead]"
}

fn project_slug_to_display(slug: &str) -> String {
    // Handle canonical _HOME_ prefix
    if slug.starts_with("_HOME_") {
        let rest = slug.strip_prefix("_HOME_").unwrap_or("");
        let rest = rest.trim_start_matches('-');
        if rest.is_empty() {
            return "Home".to_string();
        }
        return format!("~/{}", rest.replace('-', "/"));
    }

    // Convert "-home-rayyan-pc-Downloads-Github-myproject" → "~/Downloads/Github/myproject"
    let s = slug.trim_start_matches('-');
    let parts: Vec<&str> = s.split('-').collect();

    // Try to reconstruct a readable path
    // Find "Downloads" or "home" markers
    if let Some(pos) = parts.iter().position(|&p| p.eq_ignore_ascii_case("downloads")) {
        let rest = parts[pos..].join("/");
        return format!("~/{rest}");
    }
    if let Some(pos) = parts.iter().position(|&p| p.eq_ignore_ascii_case("home")) {
        let rest = parts[pos + 2..].join("/"); // skip "home" and username
        if rest.is_empty() {
            return "Home".to_string();
        }
        return format!("~/{rest}");
    }

    // Fallback: just use last 2 dash-separated segments
    let meaningful: Vec<&str> = parts.iter().filter(|p| !p.is_empty()).cloned().collect();
    if meaningful.len() >= 2 {
        meaningful[meaningful.len() - 2..].join("/")
    } else {
        slug.to_string()
    }
}

fn parse_session(path: &std::path::Path, project_slug: &str) -> Result<Option<ChatSession>> {
    let content = fs::read_to_string(path)?;
    let file_size_bytes = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let line_count = content.lines().count();

    // Check if this file exists in the sync repo
    let is_synced = {
        let sync_repo = crate::sync::engine::sync_repo_path();
        if sync_repo.exists() {
            let rel = path.strip_prefix(super::claude_dir()).unwrap_or(path);
            let canonical = super::canonicalize_file_key(&super::normalize_path(&rel.to_string_lossy()));
            sync_repo.join(&canonical).exists()
        } else {
            false
        }
    };

    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut first_user_msg = String::new();
    let mut message_count = 0;
    let mut earliest_ts = String::new();

    for line in content.lines() {
        let Ok(entry) = serde_json::from_str::<RawEntry>(line) else {
            continue;
        };

        // Skip meta, sidechain, snapshots
        if entry.is_meta.unwrap_or(false) || entry.is_sidechain.unwrap_or(false) {
            continue;
        }
        let etype = entry.entry_type.as_deref().unwrap_or("");
        if etype == "file-history-snapshot" || etype == "summary" {
            continue;
        }

        let Some(msg) = &entry.message else { continue };
        let role = msg.role.as_deref().unwrap_or("");
        if role != "user" && role != "assistant" {
            continue;
        }

        let content_str = msg
            .content
            .as_ref()
            .map(extract_text)
            .unwrap_or_default();

        if is_noise(&content_str) {
            continue;
        }

        if earliest_ts.is_empty() {
            earliest_ts = entry.timestamp.clone().unwrap_or_default();
        }

        message_count += 1;

        if first_user_msg.is_empty() && role == "user" {
            first_user_msg = content_str.chars().take(120).collect();
        }
    }

    if message_count == 0 {
        return Ok(None);
    }

    Ok(Some(ChatSession {
        id,
        project_slug: project_slug.to_string(),
        project_display: project_slug_to_display(project_slug),
        timestamp: earliest_ts,
        message_count,
        first_user_message: first_user_msg,
        path: super::normalize_path(&path.to_string_lossy()),
        file_size_bytes,
        line_count,
        is_synced,
    }))
}

pub fn list_sessions() -> Result<Vec<ChatSession>> {
    let projects_dir = projects_dir();
    if !projects_dir.exists() {
        return Ok(vec![]);
    }

    let mut sessions = vec![];

    for entry in WalkDir::new(&projects_dir)
        .max_depth(2)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| ext == "jsonl")
                && !e
                    .path()
                    .to_string_lossy()
                    .contains("/subagents/")
        })
    {
        // project slug is the direct child of projects_dir
        let rel = entry.path().strip_prefix(&projects_dir).unwrap_or(entry.path());
        let project_slug = rel
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
            .unwrap_or("unknown");

        if let Ok(Some(session)) = parse_session(entry.path(), project_slug) {
            sessions.push(session);
        }
    }

    // Sort newest first
    sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(sessions)
}

/// Delete a single chat session locally, and optionally from the sync repo.
pub fn delete_session(session_path: &str, delete_from_sync: bool) -> Result<()> {
    let path = PathBuf::from(super::native_path(session_path));
    if path.exists() {
        fs::remove_file(&path)?;
    }

    if delete_from_sync {
        let sync_repo = crate::sync::engine::sync_repo_path();
        if sync_repo.exists() {
            let claude_dir = super::claude_dir();
            if let Ok(rel) = path.strip_prefix(&claude_dir) {
                let canonical = super::canonicalize_file_key(&super::normalize_path(&rel.to_string_lossy()));
                let sync_path = sync_repo.join(&canonical);
                if sync_path.exists() {
                    fs::remove_file(&sync_path)?;
                }
            }
        }
    }

    Ok(())
}

/// Delete multiple chat sessions. Returns list of successfully deleted paths.
pub fn delete_sessions(paths: Vec<String>, delete_from_sync: bool) -> Result<Vec<String>> {
    let mut deleted = vec![];
    for path in &paths {
        if delete_session(path, delete_from_sync).is_ok() {
            deleted.push(path.clone());
        }
    }
    Ok(deleted)
}

pub fn get_session_messages(session_path: &str) -> Result<Vec<ChatMessage>> {
    let path = PathBuf::from(super::native_path(session_path));
    let content = fs::read_to_string(&path)?;
    let mut messages = vec![];

    for line in content.lines() {
        let Ok(entry) = serde_json::from_str::<RawEntry>(line) else {
            continue;
        };

        if entry.is_meta.unwrap_or(false) || entry.is_sidechain.unwrap_or(false) {
            continue;
        }
        let etype = entry.entry_type.as_deref().unwrap_or("");
        if etype == "file-history-snapshot" || etype == "summary" {
            continue;
        }

        let Some(msg) = &entry.message else { continue };
        let role = msg.role.as_deref().unwrap_or("");
        if role != "user" && role != "assistant" {
            continue;
        }

        let content_str = msg
            .content
            .as_ref()
            .map(extract_text)
            .unwrap_or_default();

        if is_noise(&content_str) {
            continue;
        }

        let is_tool_use = content_str.starts_with("[Tool:");

        messages.push(ChatMessage {
            role: role.to_string(),
            content: content_str,
            timestamp: entry.timestamp.clone().unwrap_or_default(),
            is_tool_use,
        });
    }

    Ok(messages)
}
