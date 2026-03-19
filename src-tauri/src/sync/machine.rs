use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPaths {
    /// Override for ~/.claude (the entire claude dir)
    #[serde(rename = "claudeDir", skip_serializing_if = "Option::is_none")]
    pub claude_dir: Option<String>,
    /// Override for agents directory (default: <claudeDir>/agents)
    #[serde(rename = "agentsDir", skip_serializing_if = "Option::is_none")]
    pub agents_dir: Option<String>,
    /// Override for skills directory (default: <claudeDir>/skills)
    #[serde(rename = "skillsDir", skip_serializing_if = "Option::is_none")]
    pub skills_dir: Option<String>,
    /// Override for projects directory (default: <claudeDir>/projects)
    #[serde(rename = "projectsDir", skip_serializing_if = "Option::is_none")]
    pub projects_dir: Option<String>,
}

impl Default for CustomPaths {
    fn default() -> Self {
        Self {
            claude_dir: None,
            agents_dir: None,
            skills_dir: None,
            projects_dir: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    #[serde(rename = "machineId")]
    pub machine_id: String,
    #[serde(rename = "machineName")]
    pub machine_name: String,
    #[serde(rename = "remoteUrl")]
    pub remote_url: Option<String>,
    #[serde(rename = "autoSyncInterval")]
    pub auto_sync_interval: u64,
    #[serde(rename = "machineOverrides")]
    pub machine_overrides: Vec<String>,
    #[serde(rename = "lastSynced")]
    pub last_synced: Option<String>,
    #[serde(rename = "customPaths", default)]
    pub custom_paths: CustomPaths,
}

impl Default for MachineConfig {
    fn default() -> Self {
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "My Machine".to_string());

        Self {
            machine_id: Uuid::new_v4().to_string(),
            machine_name: hostname,
            remote_url: None,
            auto_sync_interval: 15,
            machine_overrides: vec![],
            last_synced: None,
            custom_paths: CustomPaths::default(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".claude-sync")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn hashes_path() -> PathBuf {
    config_dir().join("hashes.json")
}

pub async fn ensure_machine_config() -> Result<MachineConfig> {
    let path = config_path();

    if path.exists() {
        return read_machine_config().await;
    }

    let config = MachineConfig::default();
    write_machine_config(&config).await?;
    Ok(config)
}

pub async fn read_machine_config() -> Result<MachineConfig> {
    let path = config_path();

    if !path.exists() {
        return Ok(MachineConfig::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;

    let config: MachineConfig = serde_json::from_str(&content)
        .with_context(|| "Failed to parse machine config")?;

    Ok(config)
}

pub async fn write_machine_config(config: &MachineConfig) -> Result<()> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)
        .with_context(|| format!("Failed to write config: {}", path.display()))?;

    Ok(())
}

