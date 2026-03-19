use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::machine::hashes_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHashes {
    pub hashes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictStatus {
    Unchanged,
    LocalOnly,
    RemoteOnly,
    Conflict,
}

pub fn hash_file(path: &Path) -> Result<String> {
    let content = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(hex::encode(hasher.finalize()))
}

pub fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

pub fn load_hashes() -> FileHashes {
    let path = hashes_path();

    if !path.exists() {
        return FileHashes {
            hashes: HashMap::new(),
        };
    }

    let content = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or(FileHashes {
        hashes: HashMap::new(),
    })
}

pub fn save_hashes(hashes: &FileHashes) -> Result<()> {
    let path = hashes_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(hashes)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn update_hash(hashes: &mut FileHashes, file_key: &str, hash: &str) {
    hashes.hashes.insert(file_key.to_string(), hash.to_string());
}

/// Detect conflict status for a file
/// - local_path: path to the local file
/// - remote_content: content of remote file (after fetch)
/// - file_key: normalized key used in hashes store
pub fn detect_conflict(
    local_path: &Path,
    remote_content: Option<&[u8]>,
    file_key: &str,
) -> ConflictStatus {
    let hashes = load_hashes();
    let stored_hash = hashes.hashes.get(file_key).cloned();

    let local_hash = if local_path.exists() {
        hash_file(local_path).ok()
    } else {
        None
    };

    let remote_hash = remote_content.map(hash_content);

    match (stored_hash, local_hash, remote_hash) {
        // Both unchanged
        (Some(stored), Some(local), Some(remote))
            if local == stored && remote == stored =>
        {
            ConflictStatus::Unchanged
        }
        // Only local changed
        (Some(stored), Some(local), Some(remote))
            if local != stored && remote == stored =>
        {
            ConflictStatus::LocalOnly
        }
        // Only remote changed
        (Some(stored), Some(local), Some(remote))
            if local == stored && remote != stored =>
        {
            ConflictStatus::RemoteOnly
        }
        // Both changed — conflict
        (Some(stored), Some(local), Some(remote))
            if local != stored && remote != stored =>
        {
            ConflictStatus::Conflict
        }
        // No stored hash — treat as remote-only pull
        (None, _, Some(_)) => ConflictStatus::RemoteOnly,
        // No stored hash, local exists — treat as local-only push
        (None, Some(_), None) => ConflictStatus::LocalOnly,
        _ => ConflictStatus::Unchanged,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Resolution {
    Mine,
    Theirs,
    Manual(String),
}

pub fn apply_resolution(
    local_path: &Path,
    remote_content: &[u8],
    resolution: &Resolution,
) -> Result<String> {
    match resolution {
        Resolution::Mine => {
            // Keep local — do nothing to file, just return local content
            let content = fs::read_to_string(local_path).unwrap_or_default();
            Ok(content)
        }
        Resolution::Theirs => {
            // Write remote content to local
            fs::write(local_path, remote_content)?;
            Ok(String::from_utf8_lossy(remote_content).to_string())
        }
        Resolution::Manual(content) => {
            fs::write(local_path, content.as_bytes())?;
            Ok(content.clone())
        }
    }
}
