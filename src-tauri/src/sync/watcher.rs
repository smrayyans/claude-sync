use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::claude;

const DEBOUNCE_MS: u64 = 1500;
const SAFE_SYNC_DELAY_SECS: u64 = 3;

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    pub receiver: mpsc::Receiver<Vec<PathBuf>>,
}

struct DebounceState {
    pending: HashMap<PathBuf, Instant>,
}

impl FileWatcher {
    pub fn new() -> Result<Self> {
        let (event_tx, mut event_rx) = mpsc::channel::<Event>(100);
        let (batch_tx, batch_rx) = mpsc::channel::<Vec<PathBuf>>(10);

        // Spawn debounce task
        tokio::spawn(async move {
            let state = Arc::new(Mutex::new(DebounceState {
                pending: HashMap::new(),
            }));

            let flush_state = state.clone();
            let flush_tx = batch_tx.clone();

            // Flush timer
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(DEBOUNCE_MS)).await;
                    let ready: Vec<PathBuf> = {
                        let mut s = flush_state.lock().unwrap();
                        let now = Instant::now();
                        let ready: Vec<PathBuf> = s
                            .pending
                            .iter()
                            .filter(|(_, t)| now.duration_since(**t).as_millis() >= DEBOUNCE_MS as u128)
                            .map(|(p, _)| p.clone())
                            .collect();
                        for p in &ready {
                            s.pending.remove(p);
                        }
                        ready
                        // MutexGuard dropped here before any await
                    };

                    if !ready.is_empty() {
                        let _ = flush_tx.send(ready).await;
                    }
                }
            });

            while let Some(event) = event_rx.recv().await {
                if matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                ) {
                    let mut s = state.lock().unwrap();
                    for path in event.paths {
                        s.pending.insert(path, Instant::now());
                    }
                }
            }
        });

        let (tx, _) = mpsc::channel::<Event>(100);
        let tx2 = event_tx;

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    let _ = tx2.blocking_send(event);
                }
            },
            Config::default(),
        )?;

        // Watch Claude directories
        let claude_dir = claude::claude_dir();
        let agents_dir = claude::agents_dir();
        let projects_dir = claude::projects_dir();

        for dir in [&claude_dir, &agents_dir, &projects_dir] {
            if dir.exists() {
                watcher.watch(dir, RecursiveMode::Recursive)?;
            }
        }

        Ok(FileWatcher {
            _watcher: watcher,
            receiver: batch_rx,
        })
    }
}

/// Check if a file was recently modified (within seconds) — safe sync check
pub fn is_recently_modified(path: &std::path::Path, within_secs: u64) -> bool {
    if let Ok(meta) = std::fs::metadata(path) {
        if let Ok(modified) = meta.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                return elapsed.as_secs() < within_secs;
            }
        }
    }
    false
}

/// Wait until a file is stable (not being written by Claude)
pub async fn wait_for_stable(path: &std::path::Path) {
    let mut retries = 0;
    while retries < 3 {
        if is_recently_modified(path, SAFE_SYNC_DELAY_SECS) {
            log::debug!("Waiting for file to stabilize: {}", path.display());
            tokio::time::sleep(Duration::from_secs(SAFE_SYNC_DELAY_SECS)).await;
            retries += 1;
        } else {
            break;
        }
    }
}
