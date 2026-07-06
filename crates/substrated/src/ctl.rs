//! The control socket: a newline-JSON request/reply channel so the CLI can
//! query the running daemon (using its warm cache and the single writer) off
//! the broadcast bus. When no daemon is up, the CLI falls back to opening the
//! DB directly (see `cli`).

use crate::daemon::{Command, DaemonState};
use crate::{db, search};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    Query {
        path: String,
        top: usize,
        time_aware: bool,
    },
    List {
        limit: usize,
    },
    Stats,
    Reindex,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reply {
    Query(Vec<search::QueryHit>),
    List(Vec<search::QueryHit>),
    Stats(db::Stats),
    Report(db::Stats),
    Error(String),
}

/// Serve the control socket until `shutdown`. Non-blocking accept so shutdown
/// is observed promptly.
pub fn serve(
    path: &Path,
    state: Arc<DaemonState>,
    cmd_tx: SyncSender<Command>,
    shutdown: &'static AtomicBool,
) {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }
    let _ = std::fs::remove_file(path);
    let listener = match UnixListener::bind(path) {
        Ok(l) => l,
        Err(e) => {
            crate::log(
                "substrated:ctl",
                &format!("bind {} failed: {e}", path.display()),
            );
            return;
        }
    };
    listener.set_nonblocking(true).ok();
    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                let state = state.clone();
                let cmd_tx = cmd_tx.clone();
                std::thread::spawn(move || handle(stream, state, cmd_tx));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(_) => break,
        }
    }
    let _ = std::fs::remove_file(path);
}

fn handle(stream: UnixStream, state: Arc<DaemonState>, cmd_tx: SyncSender<Command>) {
    let mut writer = match stream.try_clone() {
        Ok(w) => w,
        Err(_) => return,
    };
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Request>(&line) {
            Ok(req) => dispatch(req, &state, &cmd_tx),
            Err(e) => Reply::Error(format!("bad request: {e}")),
        };
        let mut buf = match serde_json::to_vec(&reply) {
            Ok(b) => b,
            Err(_) => break,
        };
        buf.push(b'\n');
        if writer.write_all(&buf).is_err() {
            break;
        }
    }
}

fn dispatch(req: Request, state: &Arc<DaemonState>, cmd_tx: &SyncSender<Command>) -> Reply {
    match run_request(req, state, cmd_tx) {
        Ok(r) => r,
        Err(e) => Reply::Error(e.to_string()),
    }
}

fn run_request(
    req: Request,
    state: &Arc<DaemonState>,
    cmd_tx: &SyncSender<Command>,
) -> Result<Reply> {
    match req {
        Request::Query {
            path,
            top,
            time_aware,
        } => {
            let conn = db::open_ro(&state.db_path)?;
            let cache = state.cache.read().unwrap().clone();
            let hits = search::query_neighbors(&conn, &cache, &path, top, time_aware)?;
            Ok(Reply::Query(hits))
        }
        Request::List { limit } => {
            let conn = db::open_ro(&state.db_path)?;
            let hits = search::briefs_to_hits(db::list_items(&conn, limit)?);
            Ok(Reply::List(hits))
        }
        Request::Stats => {
            let conn = db::open_ro(&state.db_path)?;
            Ok(Reply::Stats(db::stats(&conn)?))
        }
        Request::Reindex => {
            cmd_tx.send(Command::Reindex).ok();
            // Barrier: wait for the indexer to drain up to here.
            let (ack_tx, ack_rx) = sync_channel(1);
            cmd_tx.send(Command::Flush(ack_tx)).ok();
            let _ = ack_rx.recv_timeout(Duration::from_secs(300));
            let conn = db::open_ro(&state.db_path)?;
            Ok(Reply::Report(db::stats(&conn)?))
        }
    }
}

/// CLI-side: send one request to the daemon and read one reply. Returns
/// `Ok(None)` if no daemon is listening (so the caller can fall back to a
/// direct DB read).
pub fn request(path: &Path, req: &Request) -> Result<Option<Reply>> {
    let mut stream = match UnixStream::connect(path) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };
    stream.set_read_timeout(Some(Duration::from_secs(310)))?;
    let mut buf = serde_json::to_vec(req)?;
    buf.push(b'\n');
    stream.write_all(&buf)?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    if line.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(line.trim())?))
}
