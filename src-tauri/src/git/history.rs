use anyhow::Result;
use git2::Repository;

use super::Commit;

pub fn get_history(repo: &Repository, limit: usize) -> Result<Vec<Commit>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commits = vec![];

    for oid in revwalk.take(limit) {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        let message = commit.message().unwrap_or("").to_string();
        let machine_name = extract_machine_name(&message);

        let timestamp = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();

        let hash = oid.to_string();
        let short_hash = hash[..7].to_string();

        let files_changed = count_files_changed(repo, &commit);

        commits.push(Commit {
            hash,
            short_hash,
            message: message.lines().next().unwrap_or("").to_string(),
            author: commit.author().name().unwrap_or("unknown").to_string(),
            timestamp,
            machine_name,
            files_changed,
        });
    }

    Ok(commits)
}

fn extract_machine_name(message: &str) -> Option<String> {
    // Message format: [MachineName] sync: ...
    let start = message.find('[')?;
    let end = message.find(']')?;
    if start < end {
        Some(message[start + 1..end].to_string())
    } else {
        None
    }
}

fn count_files_changed(repo: &Repository, commit: &git2::Commit) -> usize {
    let tree = match commit.tree() {
        Ok(t) => t,
        Err(_) => return 0,
    };

    let parent_tree = commit
        .parent(0)
        .ok()
        .and_then(|p| p.tree().ok());

    let diff = match repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None) {
        Ok(d) => d,
        Err(_) => return 0,
    };

    diff.stats()
        .map(|s| s.files_changed())
        .unwrap_or(0)
}

pub fn get_commit_diff(repo: &Repository, hash: &str) -> Result<String> {
    let oid = git2::Oid::from_str(hash)?;
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;

    let parent_tree = commit
        .parent(0)
        .ok()
        .and_then(|p| p.tree().ok());

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

    let mut output = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let prefix = match line.origin() {
            '+' => "+",
            '-' => "-",
            ' ' => " ",
            _ => "",
        };
        output.push_str(prefix);
        if let Ok(s) = std::str::from_utf8(line.content()) {
            output.push_str(s);
        }
        true
    })?;

    Ok(output)
}
