pub mod agents;
pub mod history;
pub mod memory;
pub mod settings;

use std::path::PathBuf;

fn resolve_path(env_var: &str, default_fn: impl Fn() -> PathBuf) -> PathBuf {
    // Check env var override (set by the app from config on startup)
    if let Ok(p) = std::env::var(env_var) {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    default_fn()
}

/// Returns the ~/.claude directory (or custom override)
pub fn claude_dir() -> PathBuf {
    resolve_path("CLAUDE_SYNC_CLAUDE_DIR", || {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".claude")
    })
}

/// Returns the agents directory (or custom override)
pub fn agents_dir() -> PathBuf {
    resolve_path("CLAUDE_SYNC_AGENTS_DIR", || claude_dir().join("agents"))
}

/// Returns the skills directory (or custom override)
pub fn skills_dir() -> PathBuf {
    resolve_path("CLAUDE_SYNC_SKILLS_DIR", || claude_dir().join("skills"))
}

/// Returns the projects directory (or custom override)
pub fn projects_dir() -> PathBuf {
    resolve_path("CLAUDE_SYNC_PROJECTS_DIR", || claude_dir().join("projects"))
}

/// Normalize path separators to forward slashes for cross-platform storage
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Restore native path separators for the current OS
#[cfg(windows)]
pub fn native_path(path: &str) -> String {
    path.replace('/', "\\")
}

#[cfg(not(windows))]
pub fn native_path(path: &str) -> String {
    path.to_string()
}

/// Apply custom paths from MachineConfig to env vars so all modules pick them up
pub fn apply_custom_paths(config: &crate::sync::machine::MachineConfig) {
    let p = &config.custom_paths;
    if let Some(ref d) = p.claude_dir {
        std::env::set_var("CLAUDE_SYNC_CLAUDE_DIR", d);
    }
    if let Some(ref d) = p.agents_dir {
        std::env::set_var("CLAUDE_SYNC_AGENTS_DIR", d);
    }
    if let Some(ref d) = p.skills_dir {
        std::env::set_var("CLAUDE_SYNC_SKILLS_DIR", d);
    }
    if let Some(ref d) = p.projects_dir {
        std::env::set_var("CLAUDE_SYNC_PROJECTS_DIR", d);
    }
}
