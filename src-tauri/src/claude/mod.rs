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

/// The canonical placeholder for the home directory in project path names.
/// Claude Code encodes project paths as: /home/user/foo/bar -> -home-user-foo-bar
/// We replace the home-dir prefix with this so project data is machine-agnostic.
const HOME_PLACEHOLDER: &str = "_HOME_";

/// Get the home directory encoded as Claude Code does it (e.g. "home-rayyan-pc")
fn encoded_home_prefix() -> String {
    let home = dirs::home_dir().expect("Could not find home directory");
    // /home/rayyan-pc -> home-rayyan-pc
    let encoded = home.to_string_lossy()
        .trim_start_matches('/')
        .replace('/', "-");
    encoded
}

/// Convert a local project dir name to a canonical (machine-agnostic) form.
/// e.g. "-home-rayyan-pc-Downloads-Github" -> "_HOME_-Downloads-Github"
pub fn canonicalize_project_dir(dir_name: &str) -> String {
    let home_prefix = encoded_home_prefix();
    let stripped = dir_name.trim_start_matches('-');
    if stripped.starts_with(&home_prefix) {
        let rest = &stripped[home_prefix.len()..];
        format!("{HOME_PLACEHOLDER}{rest}")
    } else {
        dir_name.to_string()
    }
}

/// Convert a canonical project dir name to the local machine's form.
/// e.g. "_HOME_-Downloads-Github" -> "-home-rayyan-laptop-Downloads-Github"
pub fn localize_project_dir(canonical_name: &str) -> String {
    if canonical_name.starts_with(HOME_PLACEHOLDER) {
        let rest = &canonical_name[HOME_PLACEHOLDER.len()..];
        let home_prefix = encoded_home_prefix();
        format!("-{home_prefix}{rest}")
    } else {
        canonical_name.to_string()
    }
}

/// Remap a file_key's project directory from local to canonical form.
/// "projects/-home-rayyan-pc-Downloads-Github/memory/MEMORY.md"
///   -> "projects/_HOME_-Downloads-Github/memory/MEMORY.md"
pub fn canonicalize_file_key(key: &str) -> String {
    if let Some(rest) = key.strip_prefix("projects/") {
        if let Some(slash_pos) = rest.find('/') {
            let dir_name = &rest[..slash_pos];
            let remainder = &rest[slash_pos..];
            let canonical = canonicalize_project_dir(dir_name);
            return format!("projects/{canonical}{remainder}");
        }
    }
    key.to_string()
}

/// Remap a file_key's project directory from canonical to local form.
/// "projects/_HOME_-Downloads-Github/memory/MEMORY.md"
///   -> "projects/-home-rayyan-laptop-Downloads-Github/memory/MEMORY.md"
pub fn localize_file_key(key: &str) -> String {
    if let Some(rest) = key.strip_prefix("projects/") {
        if let Some(slash_pos) = rest.find('/') {
            let dir_name = &rest[..slash_pos];
            let remainder = &rest[slash_pos..];
            let local = localize_project_dir(dir_name);
            return format!("projects/{local}{remainder}");
        }
    }
    key.to_string()
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
