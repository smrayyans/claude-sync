use std::collections::{HashMap, HashSet};

pub enum MergeResult {
    /// Merge succeeded automatically
    AutoMerged(Vec<u8>),
    /// Conflict: both versions preserved, user must decide
    Conflict { local: Vec<u8>, remote: Vec<u8> },
}

/// Dispatch to the right merge strategy based on file type.
/// Called when both local and remote have changed (ConflictStatus::Conflict).
pub fn smart_merge(file_key: &str, local: &[u8], remote: &[u8]) -> MergeResult {
    // Identical content = no conflict
    if local == remote {
        return MergeResult::AutoMerged(local.to_vec());
    }

    // .jsonl chat history: always auto-merge by appending unique lines
    if file_key.ends_with(".jsonl") {
        return MergeResult::AutoMerged(merge_jsonl(local, remote));
    }

    // Plugin manifests: union merge
    if file_key == "plugins/installed_plugins.json" {
        return MergeResult::AutoMerged(merge_plugin_manifest(local, remote));
    }
    if file_key == "plugins/known_marketplaces.json"
        || file_key == "plugins/blocklist.json"
    {
        return MergeResult::AutoMerged(merge_json_objects(local, remote));
    }

    // settings.json: handled separately by machine overrides system
    if file_key == "settings.json" {
        return MergeResult::AutoMerged(merge_json_objects(local, remote));
    }

    // Memory .md files: NEVER auto-resolve
    if file_key.contains("/memory/") && file_key.ends_with(".md") {
        return MergeResult::Conflict {
            local: local.to_vec(),
            remote: remote.to_vec(),
        };
    }

    // plans/*.md: conflict (caller will keep both with machine suffix)
    if file_key.starts_with("plans/") && file_key.ends_with(".md") {
        return MergeResult::Conflict {
            local: local.to_vec(),
            remote: remote.to_vec(),
        };
    }

    // Default: conflict
    MergeResult::Conflict {
        local: local.to_vec(),
        remote: remote.to_vec(),
    }
}

/// Merge two .jsonl files by appending unique lines.
/// Local lines come first, then new remote lines that aren't in local.
/// Deduplicates by exact line content.
pub fn merge_jsonl(local: &[u8], remote: &[u8]) -> Vec<u8> {
    let local_str = String::from_utf8_lossy(local);
    let remote_str = String::from_utf8_lossy(remote);

    let mut seen: HashSet<String> = HashSet::new();
    let mut result: Vec<String> = Vec::new();

    // Add all local lines first (preserves local order)
    for line in local_str.lines() {
        if !line.trim().is_empty() && seen.insert(line.to_string()) {
            result.push(line.to_string());
        }
    }

    // Append remote lines that aren't already present
    for line in remote_str.lines() {
        if !line.trim().is_empty() && seen.insert(line.to_string()) {
            result.push(line.to_string());
        }
    }

    let mut output = result.join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    output.into_bytes()
}

/// Merge installed_plugins.json: union of plugin entries.
/// If same plugin exists on both sides, keep the one with higher version.
fn merge_plugin_manifest(local: &[u8], remote: &[u8]) -> Vec<u8> {
    let local_val: serde_json::Value =
        serde_json::from_slice(local).unwrap_or(serde_json::Value::Object(Default::default()));
    let remote_val: serde_json::Value =
        serde_json::from_slice(remote).unwrap_or(serde_json::Value::Object(Default::default()));

    let mut merged = local_val.clone();

    if let (Some(local_obj), Some(remote_obj)) = (merged.as_object_mut(), remote_val.as_object()) {
        // Merge top-level "plugins" object
        if let Some(remote_plugins) = remote_obj.get("plugins").and_then(|v| v.as_object()) {
            let local_plugins = local_obj
                .entry("plugins")
                .or_insert_with(|| serde_json::Value::Object(Default::default()));

            if let Some(lp) = local_plugins.as_object_mut() {
                for (key, remote_entry) in remote_plugins {
                    if !lp.contains_key(key) {
                        // Plugin only on remote -- add it
                        lp.insert(key.clone(), remote_entry.clone());
                    } else {
                        // Both have it -- keep the one with higher version
                        let local_ver = extract_version(lp.get(key));
                        let remote_ver = extract_version(Some(remote_entry));
                        if remote_ver > local_ver {
                            lp.insert(key.clone(), remote_entry.clone());
                        }
                    }
                }
            }
        }

        // Merge any other top-level keys from remote
        for (key, val) in remote_obj {
            if key != "plugins" && !local_obj.contains_key(key) {
                local_obj.insert(key.clone(), val.clone());
            }
        }
    }

    serde_json::to_vec_pretty(&merged).unwrap_or_else(|_| local.to_vec())
}

/// Extract version string from a plugin entry for comparison.
fn extract_version(val: Option<&serde_json::Value>) -> String {
    val.and_then(|v| {
        // Plugin entries can be arrays (multiple scopes) or objects
        if let Some(arr) = v.as_array() {
            arr.first()
                .and_then(|e| e.get("version"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else if let Some(obj) = v.as_object() {
            obj.get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    })
    .unwrap_or_default()
}

/// Generic JSON object merge: union of all keys from both sides.
/// If same key exists, local wins (preserves local machine's settings).
fn merge_json_objects(local: &[u8], remote: &[u8]) -> Vec<u8> {
    let local_val: serde_json::Value =
        serde_json::from_slice(local).unwrap_or(serde_json::Value::Object(Default::default()));
    let remote_val: serde_json::Value =
        serde_json::from_slice(remote).unwrap_or(serde_json::Value::Object(Default::default()));

    let mut merged = remote_val.clone();

    // Local values overwrite remote (local wins)
    if let (Some(merged_obj), Some(local_obj)) = (merged.as_object_mut(), local_val.as_object()) {
        for (key, val) in local_obj {
            merged_obj.insert(key.clone(), val.clone());
        }
    }

    serde_json::to_vec_pretty(&merged).unwrap_or_else(|_| local.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_jsonl_dedup() {
        let local = b"line1\nline2\nline3\n";
        let remote = b"line2\nline3\nline4\n";
        let merged = merge_jsonl(local, remote);
        let result = String::from_utf8(merged).unwrap();
        assert_eq!(result, "line1\nline2\nline3\nline4\n");
    }

    #[test]
    fn test_merge_jsonl_no_loss() {
        let local = b"a\nb\nc\n";
        let remote = b"d\ne\nf\n";
        let merged = merge_jsonl(local, remote);
        let result = String::from_utf8(merged).unwrap();
        assert!(result.contains("a\n"));
        assert!(result.contains("f\n"));
        assert_eq!(result.lines().count(), 6);
    }

    #[test]
    fn test_smart_merge_jsonl_auto() {
        let result = smart_merge("projects/_HOME_/chat.jsonl", b"a\n", b"b\n");
        assert!(matches!(result, MergeResult::AutoMerged(_)));
    }

    #[test]
    fn test_smart_merge_memory_conflict() {
        let result = smart_merge(
            "projects/_HOME_-Downloads/memory/MEMORY.md",
            b"local content",
            b"remote content",
        );
        assert!(matches!(result, MergeResult::Conflict { .. }));
    }

    #[test]
    fn test_smart_merge_identical() {
        let result = smart_merge("anything.txt", b"same", b"same");
        assert!(matches!(result, MergeResult::AutoMerged(_)));
    }
}
