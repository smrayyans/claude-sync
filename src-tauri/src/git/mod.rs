pub mod auth;
pub mod history;
pub mod repo;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
    pub machine_name: Option<String>,
    pub files_changed: usize,
}
