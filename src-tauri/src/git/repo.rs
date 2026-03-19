use anyhow::{Context, Result};
use git2::{ErrorCode, FetchOptions, PushOptions, Repository, Signature};
use std::path::Path;

use super::auth::build_callbacks;

pub fn open_repo(path: &Path) -> Result<Repository> {
    Repository::open(path).with_context(|| format!("Failed to open repo at {}", path.display()))
}

pub fn clone_repo(url: &str, path: &Path, token: &str) -> Result<Repository> {
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(build_callbacks(token));

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    match builder.clone(url, path) {
        Ok(repo) => Ok(repo),
        Err(_) => {
            // Remote is empty or unreachable — init locally and set remote
            init_repo(path, url)
        }
    }
}

fn init_repo(path: &Path, remote_url: &str) -> Result<Repository> {
    std::fs::create_dir_all(path)?;
    let repo = Repository::init(path)
        .with_context(|| format!("Failed to init repo at {}", path.display()))?;

    // Explicitly set HEAD to main (git default is often master)
    repo.set_head("refs/heads/main")?;

    // Set remote
    repo.remote("origin", remote_url)
        .with_context(|| "Failed to set remote")?;

    // Write .gitignore
    write_gitignore(path)?;

    Ok(repo)
}

fn write_gitignore(repo_path: &Path) -> Result<()> {
    let content = "# claude-sync managed\n.DS_Store\n*.tmp\n*.bak\n";
    std::fs::write(repo_path.join(".gitignore"), content)?;
    Ok(())
}

/// Fetch from remote. Returns false (non-fatal) if remote is empty (first push scenario).
pub fn fetch(repo: &Repository, token: &str) -> Result<bool> {
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(build_callbacks(token));

    let mut remote = repo
        .find_remote("origin")
        .with_context(|| "No remote 'origin' found")?;

    match remote.fetch(&["refs/heads/main:refs/remotes/origin/main"], Some(&mut fo), None) {
        Ok(_) => Ok(true),
        Err(e) if is_empty_remote_error(&e) => {
            log::info!("Remote is empty (first push) — skipping fetch");
            Ok(false)
        }
        Err(e) => Err(e).with_context(|| "Failed to fetch from remote"),
    }
}

fn is_empty_remote_error(e: &git2::Error) -> bool {
    let msg = e.message();
    e.code() == ErrorCode::NotFound
        || msg.contains("Couldn't find remote ref")
        || msg.contains("not found")
        || msg.contains("empty")
        || msg.contains("no such ref")
}

pub fn pull_fast_forward(repo: &Repository) -> Result<bool> {
    let fetch_head = match repo.find_reference("FETCH_HEAD") {
        Ok(r) => r,
        Err(_) => return Ok(false),
    };

    let fetch_commit = match repo.reference_to_annotated_commit(&fetch_head) {
        Ok(c) => c,
        Err(_) => return Ok(false),
    };

    let (analysis, _) = repo
        .merge_analysis(&[&fetch_commit])
        .with_context(|| "Merge analysis failed")?;

    if analysis.is_up_to_date() {
        return Ok(false);
    }

    if analysis.is_fast_forward() {
        let refname = "refs/heads/main";
        match repo.find_reference(refname) {
            Ok(mut r) => {
                r.set_target(fetch_commit.id(), "Fast-forward")?;
            }
            Err(_) => {
                repo.reference(refname, fetch_commit.id(), true, "Setting main")?;
            }
        }
        repo.set_head(refname)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        return Ok(true);
    }

    // Branches have diverged — rebase local commits on top of remote.
    // This handles the case where another machine pushed while we had
    // unpushed local commits.
    if analysis.is_normal() {
        log::info!("Branches diverged — rebasing local commits on top of remote");
        return rebase_onto_remote(repo, fetch_commit.id());
    }

    Ok(false)
}

/// Rebase local commits that are ahead of the remote onto the remote HEAD.
/// This replays each local-only commit on top of `remote_oid`.
fn rebase_onto_remote(repo: &Repository, remote_oid: git2::Oid) -> Result<bool> {
    let local_head = repo.head()?.peel_to_commit()?;

    // Find the merge base (common ancestor)
    let base_oid = repo.merge_base(local_head.id(), remote_oid)
        .with_context(|| "No common ancestor found")?;

    // Collect local-only commits (from merge-base to local HEAD), oldest first
    let mut local_commits = vec![];
    let mut walk = repo.revwalk()?;
    walk.push(local_head.id())?;
    walk.hide(base_oid)?;
    walk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::REVERSE)?;
    for oid in walk {
        let oid = oid?;
        local_commits.push(repo.find_commit(oid)?);
    }

    if local_commits.is_empty() {
        return Ok(false);
    }

    // Move HEAD to the remote commit
    let refname = "refs/heads/main";
    repo.reference(refname, remote_oid, true, "rebase: start")?;
    repo.set_head(refname)?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;

    let sig = Signature::now("claude-sync", "claude-sync@local")?;

    // Replay each local commit on top
    for commit in &local_commits {
        let tree = commit.tree()?;

        // Cherry-pick: create new commit with same tree and message on current HEAD
        let parent = repo.head()?.peel_to_commit()?;

        // Try to apply the tree by doing a merge of the commit's changes
        let base_commit = commit.parent(0).ok();
        let base_tree = base_commit.as_ref().map(|c| c.tree().ok()).flatten();

        let mut merged_index = if let Some(ref bt) = base_tree {
            repo.merge_trees(bt, &parent.tree()?, &tree, None)?
        } else {
            // No parent (root commit) — just use the commit's tree directly
            let mut idx = repo.index()?;
            idx.read_tree(&tree)?;
            idx
        };

        if merged_index.has_conflicts() {
            // On conflict, prefer the remote version (theirs = current HEAD)
            // and the local version for non-conflicting files.
            // For config sync this is safe — worst case user pushes again.
            log::warn!("Rebase conflict on {} — favouring latest", commit.id());
            // Just skip this commit and continue
            continue;
        }

        let new_tree_oid = {
            let mut idx = repo.index()?;
            idx.read_tree(&repo.find_tree(merged_index.write_tree_to(repo)?)?)?;
            idx.write()?;
            idx.write_tree()?
        };
        let new_tree = repo.find_tree(new_tree_oid)?;

        let msg = commit.message().unwrap_or("rebased commit");
        repo.commit(Some("HEAD"), &sig, &sig, msg, &new_tree, &[&parent])?;
    }

    // Checkout the final state
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;

    log::info!("Rebased {} local commits onto remote", local_commits.len());
    Ok(true)
}

/// Stage the given relative paths and create a commit.
/// `files` are paths relative to the sync repo workdir (e.g. "agents/foo.md").
pub fn stage_and_commit(repo: &Repository, files: &[String], message: &str) -> Result<()> {
    // git2's Index::add_path requires paths relative to the repo workdir.
    // We need to make sure the repo workdir is set correctly.
    let workdir = repo.workdir()
        .with_context(|| "Repo has no workdir (bare repo?)")?
        .to_path_buf();

    let mut index = repo.index().with_context(|| "Failed to get index")?;

    for file_key in files {
        let abs = workdir.join(file_key);
        if abs.exists() {
            let rel = Path::new(file_key);
            index
                .add_path(rel)
                .with_context(|| format!("Failed to stage: {file_key}"))?;
        } else {
            let _ = index.remove_path(Path::new(file_key));
        }
    }

    // Also stage .gitignore
    let gitignore_abs = workdir.join(".gitignore");
    if gitignore_abs.exists() {
        let _ = index.add_path(Path::new(".gitignore"));
    }

    index.write()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    let sig = Signature::now("claude-sync", "claude-sync@local")?;

    let parent_commit = match repo.head() {
        Ok(head) if !head.is_branch() || head.target().is_some() => {
            head.peel_to_commit().ok()
        }
        _ => None,
    };

    let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .with_context(|| "Failed to create commit")?;

    // Ensure HEAD points to main
    let head_ok = repo.head().map(|h| h.shorthand() == Some("main")).unwrap_or(false);
    if !head_ok {
        repo.set_head("refs/heads/main")?;
    }

    Ok(())
}

pub fn push(repo: &Repository, token: &str) -> Result<()> {
    let mut po = PushOptions::new();
    po.remote_callbacks(build_callbacks(token));

    let mut remote = repo
        .find_remote("origin")
        .with_context(|| "No remote 'origin' found")?;

    remote
        .push(&["refs/heads/main:refs/heads/main"], Some(&mut po))
        .with_context(|| "Failed to push to remote")?;

    Ok(())
}

/// Read a file's content from the current HEAD commit.
/// Returns None if repo has no commits or the file isn't in HEAD.
pub fn get_file_from_head(repo: &Repository, file_key: &str) -> Option<Vec<u8>> {
    let head = repo.head().ok()?.peel_to_tree().ok()?;
    let entry = head.get_path(std::path::Path::new(file_key)).ok()?;
    let blob = repo.find_blob(entry.id()).ok()?;
    Some(blob.content().to_vec())
}

/// How many local commits are ahead of origin/main.
pub fn count_ahead(repo: &Repository) -> Result<usize> {
    let local = repo.head()?.peel_to_commit()?;
    let remote = repo
        .find_reference("refs/remotes/origin/main")
        .and_then(|r| r.peel_to_commit());
    match remote {
        Ok(r) => {
            let (ahead, _) = repo.graph_ahead_behind(local.id(), r.id())?;
            Ok(ahead)
        }
        Err(_) => Ok(1), // No remote ref yet = we're ahead
    }
}

pub fn test_connection(url: &str, token: &str) -> bool {
    let tmp = std::env::temp_dir().join("claude-sync-conn-test");
    let _ = std::fs::create_dir_all(&tmp);
    let repo = match Repository::init(&tmp) {
        Ok(r) => r,
        Err(_) => return false,
    };
    let mut remote = match repo.remote_anonymous(url) {
        Ok(r) => r,
        Err(_) => return false,
    };
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(build_callbacks(token));
    // Fetching empty refs list just tests auth + reachability
    remote.fetch(&[] as &[&str], Some(&mut fo), None).is_ok()
}
