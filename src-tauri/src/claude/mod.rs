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

/// Returns the plugins directory (or custom override)
pub fn plugins_dir() -> PathBuf {
    resolve_path("CLAUDE_SYNC_PLUGINS_DIR", || claude_dir().join("plugins"))
}

/// Returns the plans directory (or custom override)
pub fn plans_dir() -> PathBuf {
    resolve_path("CLAUDE_SYNC_PLANS_DIR", || claude_dir().join("plans"))
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

/// Convert ANY machine's project dir name to canonical form.
/// Detects patterns like "-home-<user>-..." or "-Users-<user>-..." from any machine.
/// e.g. "-home-rayyan-laptop-Downloads-Github" -> "_HOME_-Downloads-Github"
pub fn canonicalize_project_dir_universal(dir_name: &str) -> String {
    let stripped = dir_name.trim_start_matches('-');

    // Linux/Mac: "home-<username>-..."
    if stripped.starts_with("home-") {
        let after_home = &stripped["home-".len()..]; // "rayyan-pc-Downloads-Github"
        // The username is everything up to the next path segment.
        // Claude Code encodes paths as: /home/user/foo/bar -> -home-user-foo-bar
        // The username could contain hyphens, but home dirs are typically one segment.
        // We use a heuristic: match known patterns like "home-<word>-" or "home-<word>-<word>-"
        // Actually, the simplest approach: find where a known directory name starts
        // (Downloads, Documents, Desktop, Projects, dev, src, opt, var, etc.)
        // OR just find the first component after "home-" by checking if removing it
        // produces a valid canonical result matching our own machine's pattern.
        //
        // Safest approach: the username is everything up to where the path diverges
        // from the home directory. Since Claude Code uses the FULL path, the structure
        // after the username matches the actual directory structure.
        // We'll match "home-<anything that's not a common dir>-" greedily.

        // Strategy: try to find the username by matching against common path starts
        // that come AFTER the home dir. If none found, fall back to first hyphen-word.
        if let Some(idx) = find_path_after_username(after_home) {
            let rest = &after_home[idx..];
            return format!("{HOME_PLACEHOLDER}-{rest}");
        }
    }

    // Windows: "Users-<username>-..." or "C-Users-<username>-..."
    let win_start = if stripped.starts_with("Users-") {
        Some("Users-".len())
    } else if stripped.len() > 2 && stripped.as_bytes()[1] == b'-' && stripped[2..].starts_with("Users-") {
        Some(2 + "Users-".len())
    } else {
        None
    };
    if let Some(start) = win_start {
        let after_users = &stripped[start..];
        if let Some(idx) = find_path_after_username(after_users) {
            let rest = &after_users[idx..];
            return format!("{HOME_PLACEHOLDER}-{rest}");
        }
    }

    // Already canonical or unrecognized
    dir_name.to_string()
}

/// Find where the actual path starts after the username in a hyphen-encoded path.
/// Given "rayyan-pc-Downloads-Github", returns the index of "Downloads-Github".
/// Given "rayyan-laptop-minecraft", returns the index of "minecraft".
/// Given "rayyan-laptop", returns None (no path after username = home dir itself).
fn find_path_after_username(s: &str) -> Option<usize> {
    // Common top-level directory names that appear right after the home dir
    let markers = [
        "Downloads", "Documents", "Desktop", "Projects", "projects",
        "dev", "src", "opt", "code", "Code", "workspace", "Workspace",
        "repos", "github", "Github", "GitHub", ".config", ".local",
        "go", "rust", "node", "minecraft", "snap", "Music", "Videos",
        "Pictures", "Templates", "Public",
    ];

    // Walk through possible positions where a marker could start
    let mut pos = 0;
    for (i, ch) in s.char_indices() {
        if ch == '-' && i > 0 {
            let after = &s[i + 1..];
            for marker in &markers {
                if after.starts_with(marker) {
                    let after_marker = &after[marker.len()..];
                    // Marker must be followed by '-', end of string, or nothing
                    if after_marker.is_empty() || after_marker.starts_with('-') {
                        return Some(i + 1);
                    }
                }
            }
            pos = i + 1;
        }
    }

    // If no marker found but there's content, check if the whole thing after
    // first hyphen-segment could be a path (for single-segment usernames)
    // e.g., "rayyan-Downloads" -- "rayyan" is username, "Downloads" is path
    // But we already checked markers above, so if nothing matched, the whole
    // string might just be a username with no subdirectory.

    // Fallback: if the string is just a username (no path after home dir),
    // return Some(len) so canonical becomes "_HOME_-" (matching "_HOME_" dir)
    // Actually no -- if there's genuinely no path after username, return None
    // to signal this is just the home dir project (e.g. "projects/-home-user/")
    if pos == 0 {
        // No hyphens at all -- entire string is the username
        // The canonical form is just "_HOME_" with no suffix
        return Some(s.len());
    }

    None
}

/// Check if a project dir name belongs to the current machine (not canonical, not foreign)
pub fn is_local_project_dir(dir_name: &str) -> bool {
    let home_prefix = encoded_home_prefix();
    let stripped = dir_name.trim_start_matches('-');
    stripped.starts_with(&home_prefix)
}

/// Check if a project dir name is already in canonical form
pub fn is_canonical_project_dir(dir_name: &str) -> bool {
    dir_name.starts_with(HOME_PLACEHOLDER)
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
    if let Some(ref d) = p.plugins_dir {
        std::env::set_var("CLAUDE_SYNC_PLUGINS_DIR", d);
    }
    if let Some(ref d) = p.plans_dir {
        std::env::set_var("CLAUDE_SYNC_PLANS_DIR", d);
    }
}
