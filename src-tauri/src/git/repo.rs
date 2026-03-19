use anyhow::{Context, Result};
use git2::{FetchOptions, PushOptions, Repository, Signature};
use std::path::Path;

use super::auth::build_callbacks;

pub fn open_repo(path: &Path) -> Result<Repository> {
    Repository::open(path).with_context(|| format!("Failed to open repo at {}", path.display()))
}

pub fn clone_repo(url: &str, path: &Path, token: &str) -> Result<Repository> {
    let mut callbacks = build_callbacks(token);
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    // Initialize empty repo if remote is empty
    if let Ok(repo) = builder.clone(url, path) {
        return Ok(repo);
    }

    // Fallback: init local repo
    init_repo(path, url, token)
}

pub fn init_repo(path: &Path, remote_url: &str, _token: &str) -> Result<Repository> {
    std::fs::create_dir_all(path)?;
    let repo = Repository::init(path)
        .with_context(|| format!("Failed to init repo at {}", path.display()))?;

    // Write .gitignore
    write_gitignore(path)?;

    // Set remote
    repo.remote("origin", remote_url)
        .with_context(|| "Failed to set remote")?;

    Ok(repo)
}

fn write_gitignore(repo_path: &Path) -> Result<()> {
    let gitignore = repo_path.join(".gitignore");
    let content = "# claude-sync managed — do not remove\n\
        .DS_Store\n\
        *.tmp\n\
        *.bak\n";
    std::fs::write(gitignore, content)?;
    Ok(())
}

pub fn fetch(repo: &Repository, token: &str) -> Result<()> {
    let callbacks = build_callbacks(token);
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut remote = repo
        .find_remote("origin")
        .with_context(|| "No remote 'origin' found")?;

    remote
        .fetch(&["main"], Some(&mut fo), None)
        .with_context(|| "Failed to fetch from remote")?;

    Ok(())
}

pub fn pull_fast_forward(repo: &Repository) -> Result<bool> {
    let fetch_head = match repo.find_reference("FETCH_HEAD") {
        Ok(r) => r,
        Err(_) => return Ok(false),
    };

    let fetch_commit = repo
        .reference_to_annotated_commit(&fetch_head)
        .with_context(|| "Failed to get annotated commit from FETCH_HEAD")?;

    let (analysis, _) = repo
        .merge_analysis(&[&fetch_commit])
        .with_context(|| "Merge analysis failed")?;

    if analysis.is_fast_forward() {
        let refname = "refs/heads/main";
        match repo.find_reference(refname) {
            Ok(mut r) => {
                r.set_target(fetch_commit.id(), "Fast-forward")?;
            }
            Err(_) => {
                repo.reference(
                    refname,
                    fetch_commit.id(),
                    true,
                    "Setting main to fetch head",
                )?;
            }
        }
        repo.set_head(refname)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        return Ok(true);
    }

    if analysis.is_up_to_date() {
        return Ok(false);
    }

    Ok(false)
}

pub fn stage_and_commit(repo: &Repository, files: &[String], message: &str) -> Result<()> {
    let mut index = repo.index().with_context(|| "Failed to get index")?;

    for file in files {
        let path = Path::new(file);
        if path.exists() {
            index
                .add_path(path)
                .with_context(|| format!("Failed to stage: {file}"))?;
        } else {
            let _ = index.remove_path(path);
        }
    }

    // Also add .gitignore
    let gitignore = Path::new(".gitignore");
    if gitignore.exists() {
        let _ = index.add_path(gitignore);
    }

    index.write()?;
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;

    let sig = Signature::now("claude-sync", "claude-sync@local")?;

    let parent_commit = match repo.head() {
        Ok(head) => {
            let commit = head.peel_to_commit()?;
            Some(commit)
        }
        Err(_) => None,
    };

    let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .with_context(|| "Failed to create commit")?;

    Ok(())
}

pub fn push(repo: &Repository, token: &str) -> Result<()> {
    let callbacks = build_callbacks(token);
    let mut po = PushOptions::new();
    po.remote_callbacks(callbacks);

    let mut remote = repo
        .find_remote("origin")
        .with_context(|| "No remote 'origin' found")?;

    remote
        .push(&["refs/heads/main:refs/heads/main"], Some(&mut po))
        .with_context(|| "Failed to push to remote")?;

    Ok(())
}

pub fn test_connection(url: &str, token: &str) -> bool {
    // Try to list remote refs
    let mut remote = match git2::Remote::create_detached(url) {
        Ok(r) => r,
        Err(_) => return false,
    };

    remote.connect_auth(
        git2::Direction::Fetch,
        Some(build_callbacks(token)),
        None,
    ).is_ok()
}
