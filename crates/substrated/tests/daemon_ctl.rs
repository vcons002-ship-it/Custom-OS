//! The live two-process path: spawn the built `substrated` daemon, let it scan
//! a library and serve its control socket, then drive it with the `substrated`
//! CLI (which connects over that socket).

mod common;

use std::os::unix::net::UnixListener;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_substrated");

/// A throwaway bus so the daemon's BusClient::connect succeeds. It ECHOES
/// every frame back to the sender — mirroring the real weaved bus — so this
/// test exercises the client's inbound-drain path (without which the socket
/// buffer fills and the whole bus deadlocks).
fn fake_bus(path: &Path) {
    let listener = UnixListener::bind(path).unwrap();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            std::thread::spawn(move || {
                use std::io::{Read, Write};
                let mut buf = [0u8; 4096];
                while let Ok(n) = stream.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let _ = stream.write_all(&buf[..n]); // echo back to sender
                }
            });
        }
    });
}

fn cli(env: &[(&str, &str)], args: &[&str]) -> String {
    let mut c = Command::new(BIN);
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c.args(args).output().expect("run substrated cli");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn daemon_indexes_and_serves_queries_over_ctl() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("library");
    let db = tmp.path().join("substrate.db");
    let ctl = tmp.path().join("substrated.sock");
    let bus = tmp.path().join("bus.sock");
    let groups = common::populate_library(&lib, 5); // 20 images

    fake_bus(&bus);

    let env: Vec<(&str, &str)> = vec![
        ("CLADE_SUBSTRATE_DB", db.to_str().unwrap()),
        ("CLADE_LIBRARY", lib.to_str().unwrap()),
        ("CLADE_SUBSTRATE_CTL", ctl.to_str().unwrap()),
        ("CLADE_BUS", bus.to_str().unwrap()),
        ("CLADE_SUBSTRATE_DEBOUNCE_MS", "100"),
    ];

    // Launch the daemon (no args).
    let mut daemon = {
        let mut c = Command::new(BIN);
        for (k, v) in &env {
            c.env(k, v);
        }
        c.spawn().expect("spawn substrated daemon")
    };

    // Poll `stats` over the control socket until the cold scan finishes.
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut indexed = false;
    while Instant::now() < deadline {
        let out = cli(&env, &["stats"]);
        if out.contains("items 20") {
            indexed = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(
        indexed,
        "daemon should index all 20 images and report them over ctl"
    );

    // Query a red-cluster image; neighbours come back over the socket.
    let query = groups[0][0].to_str().unwrap();
    let out = cli(&env, &["query", query, "--top", "4"]);
    let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 4, "expected 4 neighbours, got:\n{out}");
    for line in &lines {
        assert!(
            line.contains("c0_"),
            "expected a red-cluster neighbour, got: {line}"
        );
    }

    // Clean shutdown via SIGTERM.
    unsafe {
        libc::kill(daemon.id() as libc::pid_t, libc::SIGTERM);
    }
    let _ = daemon.wait();
}
