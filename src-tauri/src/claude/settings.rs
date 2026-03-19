use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use super::claude_dir;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeSettings {
    #[serde(flatten)]
    pub values: serde_json::Map<String, Value>,
}

pub fn settings_path() -> PathBuf {
    claude_dir().join("settings.json")
}

pub fn read_settings() -> Result<ClaudeSettings> {
    let path = settings_path();

    if !path.exists() {
        return Ok(ClaudeSettings::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read settings: {}", path.display()))?;

    let settings: ClaudeSettings = serde_json::from_str(&content)
        .with_context(|| "Failed to parse settings.json")?;

    Ok(settings)
}

pub fn write_settings(settings: &ClaudeSettings) -> Result<()> {
    let path = settings_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(settings)?;
    fs::write(&path, content)
        .with_context(|| format!("Failed to write settings: {}", path.display()))?;

    Ok(())
}

pub fn strip_machine_overrides(
    settings: &ClaudeSettings,
    overrides: &[String],
) -> ClaudeSettings {
    let mut filtered = settings.values.clone();

    for key in overrides {
        // Support dot-notation keys like "settings.theme"
        let actual_key = key
            .strip_prefix("settings.")
            .unwrap_or(key);
        filtered.remove(actual_key);
    }

    ClaudeSettings { values: filtered }
}

pub fn merge_settings(local: &ClaudeSettings, remote: &ClaudeSettings, overrides: &[String]) -> ClaudeSettings {
    let mut merged = remote.values.clone();

    // Apply local overrides on top of remote
    for key in overrides {
        let actual_key = key
            .strip_prefix("settings.")
            .unwrap_or(key);

        if let Some(local_value) = local.values.get(actual_key) {
            merged.insert(actual_key.to_string(), local_value.clone());
        }
    }

    ClaudeSettings { values: merged }
}
