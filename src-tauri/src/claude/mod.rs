pub mod agents;
pub mod memory;
pub mod settings;

use std::path::PathBuf;

/// Returns the ~/.claude directory path
pub fn claude_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".claude")
}

/// Returns the ~/.claude/agents directory path
pub fn agents_dir() -> PathBuf {
    claude_dir().join("agents")
}

/// Returns the ~/.claude/projects directory path
pub fn projects_dir() -> PathBuf {
    claude_dir().join("projects")
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
