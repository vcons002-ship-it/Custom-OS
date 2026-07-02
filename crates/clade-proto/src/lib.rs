//! clade-proto — the shared vocabulary of Clade's mind plane.
//!
//! Every service speaks these types over the event bus that `weaved` hosts.
//!
//! Wire format (M0): newline-delimited JSON over a Unix domain socket.
//! The Cap'n Proto schema in `schemas/clade.capnp` is the M1 replacement;
//! the Rust types here are the source of truth until then, and the swap is
//! internal to this crate.

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

/// Default path of the event bus socket, inside the OS image and the dev harness.
pub const BUS_SOCKET: &str = "/run/clade/bus.sock";

/// Environment variable that overrides [`BUS_SOCKET`] (used by the host-side
/// dev harness, where /run is not ours).
pub const BUS_SOCKET_ENV: &str = "CLADE_BUS";

/// The services `weaved` supervises, in start order.
pub const SERVICES: &[&str] = &["gated", "substrated", "modeld", "capd", "cortexd"];

/// Resolve the bus socket path for this process.
pub fn bus_socket_path() -> String {
    std::env::var(BUS_SOCKET_ENV).unwrap_or_else(|_| BUS_SOCKET.to_string())
}

/// One message on the event bus.
///
/// M0 carries lifecycle traffic only; focus/intent/plan messages land with
/// their milestones (M5/M6) so the schema grows with demonstrated need.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "kebab-case")]
pub enum Event {
    /// A service announced itself after connecting.
    ServiceUp { service: String, pid: u32 },
    /// Periodic liveness signal.
    Heartbeat { service: String, uptime_secs: u64 },
    /// PID 1 finished bringing up the mind plane; the Weave may paint.
    WeaveReady,
    /// A supervised service exited and will be restarted with backoff.
    ServiceDown { service: String, code: Option<i32> },
}

impl Event {
    /// Serialize as one bus frame (JSON + newline).
    pub fn to_frame(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = serde_json::to_vec(self)?;
        buf.push(b'\n');
        Ok(buf)
    }

    /// Parse one bus frame.
    pub fn from_frame(line: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(line)?)
    }
}

/// A connected bus client: publish [`Event`]s, iterate the ones broadcast back.
pub struct BusClient {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
}

impl BusClient {
    /// Connect to the bus, retrying briefly — services race PID 1 at boot.
    pub fn connect() -> anyhow::Result<Self> {
        let path = bus_socket_path();
        let mut last_err = None;
        for _ in 0..50 {
            match UnixStream::connect(&path) {
                Ok(stream) => {
                    let reader = BufReader::new(stream.try_clone()?);
                    return Ok(Self {
                        reader,
                        writer: stream,
                    });
                }
                Err(e) => {
                    last_err = Some(e);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
        Err(anyhow::anyhow!(
            "bus at {path} not reachable: {}",
            last_err.expect("retries imply an error")
        ))
    }

    pub fn publish(&mut self, event: &Event) -> anyhow::Result<()> {
        self.writer.write_all(&event.to_frame()?)?;
        Ok(())
    }

    /// Block for the next event broadcast on the bus.
    pub fn next_event(&mut self) -> anyhow::Result<Event> {
        let mut line = String::new();
        loop {
            line.clear();
            let n = self.reader.read_line(&mut line)?;
            if n == 0 {
                return Err(anyhow::anyhow!("bus closed"));
            }
            if !line.trim().is_empty() {
                return Event::from_frame(line.trim());
            }
        }
    }
}

/// Announce-then-heartbeat loop shared by every M0 service stub.
pub fn run_service_stub(name: &str) -> anyhow::Result<()> {
    let mut bus = BusClient::connect()?;
    bus.publish(&Event::ServiceUp {
        service: name.into(),
        pid: std::process::id(),
    })?;
    let started = std::time::Instant::now();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        bus.publish(&Event::Heartbeat {
            service: name.into(),
            uptime_secs: started.elapsed().as_secs(),
        })?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_round_trip() {
        let events = [
            Event::ServiceUp {
                service: "cortexd".into(),
                pid: 42,
            },
            Event::Heartbeat {
                service: "gated".into(),
                uptime_secs: 7,
            },
            Event::WeaveReady,
            Event::ServiceDown {
                service: "modeld".into(),
                code: Some(1),
            },
        ];
        for e in events {
            let frame = e.to_frame().unwrap();
            let text = std::str::from_utf8(&frame).unwrap();
            assert!(text.ends_with('\n'));
            assert_eq!(Event::from_frame(text.trim()).unwrap(), e);
        }
    }

    #[test]
    fn wire_format_is_stable() {
        // The dev harness and integration tests grep for these shapes; keep
        // the tag names deliberate, not accidental.
        let e = Event::ServiceUp {
            service: "capd".into(),
            pid: 1,
        };
        let json = serde_json::to_string(&e).unwrap();
        assert_eq!(json, r#"{"event":"service-up","service":"capd","pid":1}"#);
    }
}
