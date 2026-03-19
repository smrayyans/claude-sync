use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use super::agents_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub content: String,
    pub path: String,
    pub frontmatter: AgentFrontmatter,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tools: Option<Vec<String>>,
    pub model: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub content: String,
}

/// Parse YAML frontmatter from a markdown file
/// Returns (frontmatter_str, body_str)
fn parse_frontmatter(content: &str) -> (Option<String>, String) {
    if !content.starts_with("---") {
        return (None, content.to_string());
    }

    let after_first = &content[3..];
    if let Some(end_idx) = after_first.find("\n---") {
        let fm = &after_first[..end_idx];
        let body = &after_first[end_idx + 4..];
        let body = body.trim_start_matches('\n').to_string();
        (Some(fm.to_string()), body)
    } else {
        (None, content.to_string())
    }
}

/// Parse agent frontmatter from YAML string
fn parse_agent_frontmatter(yaml: &str) -> AgentFrontmatter {
    let mut fm = AgentFrontmatter::default();

    for line in yaml.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
            match key {
                "name" => fm.name = Some(value),
                "description" => fm.description = Some(value),
                "model" => fm.model = Some(value),
                "color" => fm.color = Some(value),
                _ => {}
            }
        }
    }

    fm
}

pub fn list_agents() -> Result<Vec<Agent>> {
    let dir = agents_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut agents = vec![];

    for entry in WalkDir::new(&dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| ext == "md")
        })
    {
        if let Ok(agent) = read_agent(entry.path()) {
            agents.push(agent);
        }
    }

    agents.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(agents)
}

pub fn read_agent(path: &std::path::Path) -> Result<Agent> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read agent file: {}", path.display()))?;

    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let (fm_str, body) = parse_frontmatter(&content);
    let frontmatter = fm_str
        .as_deref()
        .map(parse_agent_frontmatter)
        .unwrap_or_default();

    let description = frontmatter
        .description
        .clone()
        .unwrap_or_else(|| extract_first_line(&body));

    Ok(Agent {
        name: frontmatter.name.clone().unwrap_or_else(|| name.clone()),
        description,
        content,
        path: super::normalize_path(&path.to_string_lossy()),
        frontmatter,
    })
}

fn extract_first_line(text: &str) -> String {
    text.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim_start_matches('#')
        .trim()
        .chars()
        .take(100)
        .collect()
}

pub fn save_agent(agent: &Agent) -> Result<()> {
    let dir = agents_dir();
    fs::create_dir_all(&dir)?;

    let filename = agent.name.replace(' ', "-").to_lowercase();
    let path = dir.join(format!("{filename}.md"));

    fs::write(&path, &agent.content)
        .with_context(|| format!("Failed to write agent: {}", path.display()))?;

    Ok(())
}

pub fn delete_agent(name: &str) -> Result<()> {
    let dir = agents_dir();
    let filename = name.replace(' ', "-").to_lowercase();
    let path = dir.join(format!("{filename}.md"));

    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("Failed to delete agent: {}", path.display()))?;
    }

    Ok(())
}

pub fn get_agent_path(name: &str) -> PathBuf {
    let filename = name.replace(' ', "-").to_lowercase();
    agents_dir().join(format!("{filename}.md"))
}

pub fn list_templates() -> Vec<Template> {
    vec![
        Template {
            name: "General Assistant".to_string(),
            description: "A helpful general-purpose assistant".to_string(),
            content: r#"---
name: General Assistant
description: A helpful general-purpose assistant
tools: ["Read", "Write", "Bash", "Glob", "Grep"]
---

You are a helpful assistant. Help the user with their tasks efficiently and accurately.
"#
            .to_string(),
        },
        Template {
            name: "Code Reviewer".to_string(),
            description: "Reviews code for bugs, security issues, and best practices".to_string(),
            content: r#"---
name: Code Reviewer
description: Reviews code for bugs, security issues, and best practices
tools: ["Read", "Glob", "Grep"]
---

You are an expert code reviewer. Review code for:
- Bugs and logic errors
- Security vulnerabilities
- Performance issues
- Best practices and maintainability

Provide actionable, specific feedback.
"#
            .to_string(),
        },
        Template {
            name: "Security Auditor".to_string(),
            description: "Audits code and configs for security vulnerabilities".to_string(),
            content: r#"---
name: Security Auditor
description: Audits code and configs for security vulnerabilities
tools: ["Read", "Glob", "Grep", "Bash"]
---

You are a security auditor. Analyze code, configurations, and infrastructure for:
- OWASP Top 10 vulnerabilities
- Authentication/authorization flaws
- Injection vulnerabilities
- Insecure dependencies
- Misconfigurations

Provide severity ratings and remediation steps.
"#
            .to_string(),
        },
        Template {
            name: "Documentation Writer".to_string(),
            description: "Writes and maintains technical documentation".to_string(),
            content: r#"---
name: Documentation Writer
description: Writes and maintains technical documentation
tools: ["Read", "Write", "Glob", "Grep"]
---

You are a technical documentation writer. Create clear, comprehensive documentation including:
- API references
- User guides
- Architecture overviews
- Code comments and docstrings

Write for the appropriate audience level.
"#
            .to_string(),
        },
        Template {
            name: "CTF Solver".to_string(),
            description: "Helps solve CTF challenges (authorized security practice)".to_string(),
            content: r#"---
name: CTF Solver
description: Helps solve CTF challenges in authorized CTF competitions
tools: ["Read", "Write", "Bash", "Glob", "Grep"]
---

You are a CTF challenge assistant helping with authorized competition challenges. Help analyze:
- Web application vulnerabilities (in CTF context)
- Cryptography challenges
- Reverse engineering tasks
- Forensics challenges
- Binary exploitation (in authorized CTF labs)

Always clarify this is for authorized CTF competitions only.
"#
            .to_string(),
        },
    ]
}
