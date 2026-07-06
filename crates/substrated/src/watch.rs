//! The filesystem watcher: notify (inotify) events → debounced Upsert/Remove
//! commands to the single-writer indexer. Rename is handled implicitly —
//! remove+create converge because identity is the content hash.

use crate::config;
use crate::daemon::Command;
use notify::{RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, SyncSender};
use std::time::{Duration, Instant};

/// Watch `library` recursively until `shutdown`. Coalesces bursts per path
/// over `debounce_ms` before emitting a command.
pub fn run(
    library: &Path,
    cmds: SyncSender<Command>,
    debounce_ms: u64,
    shutdown: &'static AtomicBool,
) {
    let (raw_tx, raw_rx) = channel::<notify::Result<notify::Event>>();
    let mut watcher = match notify::recommended_watcher(move |res| {
        let _ = raw_tx.send(res);
    }) {
        Ok(w) => w,
        Err(e) => {
            crate::log("substrated:watch", &format!("watcher init failed: {e}"));
            return;
        }
    };
    if let Err(e) = watcher.watch(library, RecursiveMode::Recursive) {
        crate::log(
            "substrated:watch",
            &format!("watch({}) failed: {e}", library.display()),
        );
        return;
    }

    let debounce = Duration::from_millis(debounce_ms);
    let mut pending: HashMap<PathBuf, Instant> = HashMap::new();
    while !shutdown.load(Ordering::Relaxed) {
        match raw_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                for p in event.paths {
                    if config::has_image_ext(&p) {
                        pending.insert(p, Instant::now());
                    }
                }
            }
            Ok(Err(_)) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
        let now = Instant::now();
        let ready: Vec<PathBuf> = pending
            .iter()
            .filter(|(_, t)| now.duration_since(**t) >= debounce)
            .map(|(p, _)| p.clone())
            .collect();
        for p in ready {
            pending.remove(&p);
            let cmd = if p.is_file() {
                Command::Upsert(p)
            } else {
                Command::Remove(p)
            };
            if cmds.send(cmd).is_err() {
                return; // indexer gone
            }
        }
    }
}
