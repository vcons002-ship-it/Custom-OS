//! The service supervisor: launch each mind-plane service, restart it with
//! backoff when it dies, and publish lifecycle events on the bus.

use crate::bus::BusHandle;
use crate::log;
use clade_proto::Event;
use std::collections::HashSet;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Restart backoff: 1s, 2s, 4s, 8s, capped at 16s. A service that stays up
/// for a minute earns its backoff reset.
const BACKOFF_START: Duration = Duration::from_secs(1);
const BACKOFF_CAP: Duration = Duration::from_secs(16);
const STABLE_AFTER: Duration = Duration::from_secs(60);

pub struct Supervisor {
    bus: BusHandle,
    up: Arc<Mutex<HashSet<String>>>,
    expected: usize,
}

impl Supervisor {
    pub fn new(bus: BusHandle) -> Self {
        Self {
            bus,
            up: Arc::new(Mutex::new(HashSet::new())),
            expected: 0,
        }
    }

    /// Launch `service` on its own supervision thread.
    pub fn launch(&mut self, service: &str) {
        self.expected += 1;
        let bus = self.bus.clone();
        let up = self.up.clone();
        let service = service.to_string();

        std::thread::spawn(move || {
            let mut backoff = BACKOFF_START;
            loop {
                let started = std::time::Instant::now();
                match spawn(&service) {
                    Ok(mut child) => {
                        log(
                            "supervisor",
                            &format!("{service} started (pid {})", child.id()),
                        );
                        up.lock().expect("supervisor lock").insert(service.clone());
                        let code = child.wait().ok().and_then(|s| s.code());
                        up.lock().expect("supervisor lock").remove(&service);
                        let _ = bus.broadcast(&Event::ServiceDown {
                            service: service.clone(),
                            code,
                        });
                        if started.elapsed() >= STABLE_AFTER {
                            backoff = BACKOFF_START;
                        }
                    }
                    Err(e) => log("supervisor", &format!("{service} failed to spawn: {e}")),
                }
                log(
                    "supervisor",
                    &format!("{service} restarting in {backoff:?}"),
                );
                std::thread::sleep(backoff);
                backoff = (backoff * 2).min(BACKOFF_CAP);
            }
        });
    }

    /// Block until every launched service is currently up (M0 readiness).
    pub fn await_all_up(&self) {
        loop {
            if self.up.lock().expect("supervisor lock").len() >= self.expected {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// PID 1 never returns.
    pub fn run_forever(&self) -> ! {
        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    }
}

/// Resolve and spawn a service binary.
///
/// In the image, services live on PATH (/usr/bin). In the dev harness they
/// sit next to this executable in target/{debug,release}, so try that first.
/// Children get PDEATHSIG so a dying harness never leaves orphans behind
/// (irrelevant at PID 1, which never exits).
fn spawn(service: &str) -> std::io::Result<Child> {
    use std::os::unix::process::CommandExt;

    let sibling = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(service)))
        .filter(|p| p.exists());
    let mut cmd = match sibling {
        Some(path) => Command::new(path),
        None => Command::new(service),
    };
    unsafe {
        cmd.pre_exec(|| {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
            Ok(())
        });
    }
    cmd.spawn()
}
