//! The event bus: a broadcast hub over a Unix domain socket.
//!
//! Every connected client sees every event (including its own — echoes are
//! cheap and make the bus observable with `nc -U`). M0 traffic is lifecycle
//! only; the framing lives in clade-proto so the Cap'n Proto swap at M1
//! happens in one place.

use clade_proto::Event;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};

use crate::log;

type Peers = Arc<Mutex<HashMap<u64, UnixStream>>>;

pub struct Bus {
    peers: Peers,
}

#[derive(Clone)]
pub struct BusHandle {
    peers: Peers,
}

impl Bus {
    /// Bind the socket, start the accept loop, return the hub.
    pub fn start() -> anyhow::Result<Self> {
        let path = clade_proto::bus_socket_path();
        if let Some(dir) = std::path::Path::new(&path).parent() {
            std::fs::create_dir_all(dir)?;
        }
        let _ = std::fs::remove_file(&path); // stale socket from a prior run
        let listener = UnixListener::bind(&path)?;
        log("bus", &format!("listening on {path}"));

        let peers: Peers = Arc::new(Mutex::new(HashMap::new()));
        let accept_peers = peers.clone();
        std::thread::spawn(move || accept_loop(listener, accept_peers));
        Ok(Self { peers })
    }

    pub fn handle(&self) -> BusHandle {
        BusHandle {
            peers: self.peers.clone(),
        }
    }
}

impl BusHandle {
    /// Send an event to every connected peer. Dead peers are dropped.
    pub fn broadcast(&self, event: &Event) -> anyhow::Result<()> {
        let frame = event.to_frame()?;
        let mut peers = self.peers.lock().expect("bus lock poisoned");
        peers.retain(|_, stream| stream.write_all(&frame).is_ok());
        Ok(())
    }
}

fn accept_loop(listener: UnixListener, peers: Peers) {
    let mut next_id: u64 = 0;
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let id = next_id;
        next_id += 1;

        let Ok(writer) = stream.try_clone() else {
            continue;
        };
        peers.lock().expect("bus lock poisoned").insert(id, writer);

        // Reader thread: every line a peer sends is rebroadcast to all peers.
        let peers = peers.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stream);
            for line in reader.lines() {
                let Ok(line) = line else { break };
                if line.trim().is_empty() {
                    continue;
                }
                match Event::from_frame(line.trim()) {
                    Ok(event) => {
                        if let Ok(frame) = event.to_frame() {
                            let mut peers = peers.lock().expect("bus lock poisoned");
                            peers.retain(|_, s| s.write_all(&frame).is_ok());
                        }
                        log_event(&event);
                    }
                    Err(e) => log("bus", &format!("dropped malformed frame: {e}")),
                }
            }
            peers.lock().expect("bus lock poisoned").remove(&id);
        });
    }
}

fn log_event(event: &Event) {
    match event {
        Event::ServiceUp { service, pid } => log("bus", &format!("{service} up (pid {pid})")),
        Event::ServiceDown { service, code } => {
            log("bus", &format!("{service} down (code {code:?})"))
        }
        Event::WeaveReady => log("bus", "weave-ready"),
        Event::Heartbeat { .. } => {} // liveness is for peers, not the console
    }
}
