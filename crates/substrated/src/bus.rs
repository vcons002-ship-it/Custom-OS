//! The bus thread: owns the BusClient, announces the service, heartbeats, and
//! forwards SubstrateChanged events the indexer produces. Kept separate so the
//! !Sync indexer Connection never touches the socket.

use clade_proto::{BusClient, Event};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

/// Run until `shutdown`. If no bus is reachable (standalone/dev), drains and
/// drops events so the indexer's sends never block.
pub fn run(name: &str, events: Receiver<Event>, shutdown: &'static AtomicBool) {
    let mut bus = match BusClient::connect() {
        Ok(b) => b,
        Err(_) => {
            // No bus here — keep draining so senders don't back up.
            while !shutdown.load(Ordering::Relaxed) {
                if events.recv_timeout(Duration::from_millis(500)).is_err()
                    && shutdown.load(Ordering::Relaxed)
                {
                    break;
                }
            }
            return;
        }
    };
    let _ = bus.publish(&Event::ServiceUp {
        service: name.into(),
        pid: std::process::id(),
    });
    let started = std::time::Instant::now();
    while !shutdown.load(Ordering::Relaxed) {
        match events.recv_timeout(Duration::from_secs(5)) {
            Ok(ev) => {
                let _ = bus.publish(&ev);
            }
            Err(RecvTimeoutError::Timeout) => {
                let _ = bus.publish(&Event::Heartbeat {
                    service: name.into(),
                    uptime_secs: started.elapsed().as_secs(),
                });
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}
