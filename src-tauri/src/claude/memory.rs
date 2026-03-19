use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::{normalize_path, projects_dir};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFile {
    pub path: String,
    pub name: String,
    pub content: String,
    pub project_slug: String,
    pub frontmatter: MemoryFrontmatter,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub memory_type: Option<String>,
}

fn parse_memory_frontmatter(content: &str) -> (MemoryFrontmatter, String) {
    if !content.starts_with("---") {
        return (MemoryFrontmatter::default(), content.to_string());
    }

    let after_first = &content[3..];
    if let Some(end_idx) = after_first.find("\n---") {
        let fm_str = &after_first[..end_idx];
        let body = after_first[end_idx + 4..].trim_start_matches('\n').to_string();

        let mut fm = MemoryFrontmatter::default();
        for line in fm_str.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
                match key {
                    "name" => fm.name = Some(value),
                    "description" => fm.description = Some(value),
                    "type" => fm.memory_type = Some(value),
                    _ => {}
                }
            }
        }

        (fm, body)
    } else {
        (MemoryFrontmatter::default(), content.to_string())
    }
}

pub fn list_memory_files() -> Result<Vec<MemoryFile>> {
    let projects_dir = projects_dir();
    if !projects_dir.exists() {
        return Ok(vec![]);
    }

    let mut files = vec![];

    for entry in WalkDir::new(&projects_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            // Only .md files inside memory/ subdirectories
            e.path().extension().map_or(false, |ext| ext == "md")
                && e.path()
                    .ancestors()
                    .any(|a| a.file_name().map_or(false, |n| n == "memory"))
        })
    {
        if let Ok(mem) = read_memory_file(entry.path(), &projects_dir) {
            files.push(mem);
        }
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn read_memory_file(path: &Path, projects_dir: &Path) -> Result<MemoryFile> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read memory file: {}", path.display()))?;

    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Extract project slug from path: projects/<slug>/memory/<file>
    let rel = path.strip_prefix(projects_dir).unwrap_or(path);
    let project_slug = rel
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("unknown")
        .to_string();

    let (frontmatter, _body) = parse_memory_frontmatter(&content);

    Ok(MemoryFile {
        path: normalize_path(&path.to_string_lossy()),
        name: frontmatter.name.clone().unwrap_or(name),
        content,
        project_slug,
        frontmatter,
    })
}

pub fn get_memory_file(path: &str) -> Result<MemoryFile> {
    let p = PathBuf::from(super::native_path(path));
    let projects_dir = projects_dir();
    read_memory_file(&p, &projects_dir)
}

pub fn save_memory_file(path: &str, content: &str) -> Result<()> {
    let p = PathBuf::from(super::native_path(path));

    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&p, content)
        .with_context(|| format!("Failed to write memory file: {}", p.display()))?;

    // Update MEMORY.md index if it exists
    update_memory_index(&p)?;

    Ok(())
}

pub fn delete_memory_file(path: &str) -> Result<()> {
    let p = PathBuf::from(super::native_path(path));
    if p.exists() {
        fs::remove_file(&p)
            .with_context(|| format!("Failed to delete memory file: {}", p.display()))?;
    }
    Ok(())
}

pub fn get_project_memories(project_slug: &str) -> Result<Vec<MemoryFile>> {
    let memory_dir = projects_dir().join(project_slug).join("memory");
    let projects_dir = projects_dir();

    if !memory_dir.exists() {
        return Ok(vec![]);
    }

    let mut files = vec![];

    for entry in WalkDir::new(&memory_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
    {
        if let Ok(mem) = read_memory_file(entry.path(), &projects_dir) {
            files.push(mem);
        }
    }

    Ok(files)
}

fn update_memory_index(changed_file: &Path) -> Result<()> {
    // Find the memory dir this file belongs to
    let memory_dir = changed_file.parent().unwrap_or(changed_file);

    // Check if the parent is named "memory"
    if memory_dir.file_name().map_or(true, |n| n != "memory") {
        return Ok(());
    }

    let index_path = memory_dir.join("MEMORY.md");
    if !index_path.exists() {
        return Ok(()); // Don't create it if it doesn't exist
    }

    // Re-read all files and rebuild index entries for this file
    let filename = changed_file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let content = fs::read_to_string(changed_file).unwrap_or_default();
    let (fm, _) = parse_memory_frontmatter(&content);

    if let Some(description) = fm.description {
        let link_name = changed_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(filename);
        let _entry = format!("- [{link_name}.md]({filename}) — {description}");
        // TODO: More sophisticated index update logic
    }

    Ok(())
}
