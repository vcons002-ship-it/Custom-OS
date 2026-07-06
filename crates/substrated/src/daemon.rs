//! Daemon wiring: the single-writer indexer loop plus the bus, watcher, and
//! control-socket threads. `weaved` launches `substrated` with no args, which
//! lands here.

use crate::embed::PhashHistEmbedder;
use crate::indexer::{Indexer, Outcome};
use crate::vector::CacheEntry;
use crate::{bus, config, ctl, db, watch};
use anyhow::Result;
use clade_proto::{Event, SubstrateChange};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, Sender, SyncSender};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Set by the SIGTERM/SIGINT handler; every thread polls it.
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

/// Commands to the single-writer indexer thread.
pub enum Command {
    /// Reconcile the whole library (silent — no per-item bus events).
    ScanRoot(PathBuf),
    /// A live change: (re)index one path (emits a bus event).
    Upsert(PathBuf),
    /// A live deletion: drop one path (emits a bus event).
    Remove(PathBuf),
    /// Re-scan the configured library (used by the control socket).
    Reindex,
    /// Barrier: ack once the indexer has drained to this point.
    Flush(SyncSender<()>),
}

/// Shared read state for the control-socket handlers.
pub struct DaemonState {
    pub cache: RwLock<Vec<CacheEntry>>,
    pub db_path: PathBuf,
    pub library: PathBuf,
}

/// Run the daemon until SIGTERM. Returns when the indexer loop exits.
pub fn run() -> Result<()> {
    let db_path = config::db_path();
    let library = config::library_path();
    std::fs::create_dir_all(&library).ok();

    let ix = Indexer::open(&db_path, Box::new(PhashHistEmbedder::new()))?;
    crate::log(
        "substrated",
        &format!("db={} library={}", db_path.display(), library.display()),
    );

    install_shutdown_handler();

    let (cmd_tx, cmd_rx) = sync_channel::<Command>(1024);
    let (bus_tx, bus_rx) = std::sync::mpsc::channel::<Event>();
    let state = Arc::new(DaemonState {
        cache: RwLock::new(Vec::new()),
        db_path: db_path.clone(),
        library: library.clone(),
    });

    // Bus thread.
    std::thread::spawn(move || bus::run("substrated", bus_rx, &SHUTDOWN));
    // Control-socket thread.
    {
        let state = state.clone();
        let cmd_tx = cmd_tx.clone();
        let ctl_path = config::ctl_path();
        std::thread::spawn(move || ctl::serve(&ctl_path, state, cmd_tx, &SHUTDOWN));
    }
    // Watcher thread.
    {
        let cmd_tx = cmd_tx.clone();
        let library = library.clone();
        let debounce = config::debounce_ms();
        std::thread::spawn(move || watch::run(&library, cmd_tx, debounce, &SHUTDOWN));
    }

    // Cold scan first; watch events queue behind it.
    cmd_tx.send(Command::ScanRoot(library.clone())).ok();

    // The indexer loop owns the writer Connection on this thread.
    indexer_loop(ix, cmd_rx, state, bus_tx);
    Ok(())
}

fn indexer_loop(
    mut ix: Indexer,
    rx: Receiver<Command>,
    state: Arc<DaemonState>,
    bus_tx: Sender<Event>,
) {
    refresh_cache(&ix, &state);
    while !SHUTDOWN.load(Ordering::Relaxed) {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Command::ScanRoot(root)) => {
                match ix.scan_root(&root) {
                    Ok(r) => crate::log("substrated", &format!("scan {}: {r:?}", root.display())),
                    Err(e) => crate::log("substrated", &format!("scan failed: {e}")),
                }
                refresh_cache(&ix, &state);
            }
            Ok(Command::Reindex) => {
                let lib = state.library.clone();
                let _ = ix.scan_root(&lib);
                refresh_cache(&ix, &state);
            }
            Ok(Command::Upsert(path)) => {
                let ts = ix.next_stamp();
                let outcome = ix.index_path(&path, ts);
                publish_upsert(&ix, &path, &outcome, &bus_tx);
                refresh_cache(&ix, &state);
            }
            Ok(Command::Remove(path)) => {
                let info = brief_for_path(&ix, &path);
                if ix.remove_path(&path).unwrap_or(false) {
                    if let Some((sub_id, content_hash)) = info {
                        let _ = bus_tx.send(Event::SubstrateChanged {
                            sub_id,
                            content_hash,
                            path: path.to_string_lossy().into_owned(),
                            change: SubstrateChange::Removed,
                        });
                    }
                    refresh_cache(&ix, &state);
                }
            }
            Ok(Command::Flush(ack)) => {
                let _ = ack.send(());
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    ix.checkpoint();
    crate::log("substrated", "shutdown: wal checkpointed, exiting");
}

fn refresh_cache(ix: &Indexer, state: &Arc<DaemonState>) {
    if let Ok(cache) = ix.load_cache() {
        if let Ok(mut guard) = state.cache.write() {
            *guard = cache;
        }
    }
}

/// (sub_id, content_hash) for the item a path currently maps to.
fn brief_for_path(ix: &Indexer, path: &std::path::Path) -> Option<(String, String)> {
    let conn = ix.connection();
    let (id, hash) = db::item_id_for_path(conn, &path.to_string_lossy()).ok()??;
    let brief = db::item_brief(conn, id).ok()??;
    Some((brief.sub_id, hash))
}

fn publish_upsert(ix: &Indexer, path: &std::path::Path, outcome: &Outcome, bus_tx: &Sender<Event>) {
    let change = match outcome {
        Outcome::Indexed | Outcome::Deduped => SubstrateChange::Indexed,
        Outcome::Updated => SubstrateChange::Updated,
        _ => return,
    };
    if let Some((sub_id, content_hash)) = brief_for_path(ix, path) {
        let _ = bus_tx.send(Event::SubstrateChanged {
            sub_id,
            content_hash,
            path: path.to_string_lossy().into_owned(),
            change,
        });
    }
}

/// SIGTERM/SIGINT → flip SHUTDOWN (async-signal-safe: one atomic store).
fn install_shutdown_handler() {
    extern "C" fn on_term(_sig: libc::c_int) {
        SHUTDOWN.store(true, Ordering::SeqCst);
    }
    unsafe {
        libc::signal(libc::SIGTERM, on_term as *const () as libc::sighandler_t);
        libc::signal(libc::SIGINT, on_term as *const () as libc::sighandler_t);
    }
}
