//! weaved — Clade's init.
//!
//! As PID 1 inside the OS image: mount the pseudo-filesystems, host the event
//! bus, supervise the mind-plane services, reap orphans, print the banner,
//! and announce `weave-ready`. There is no login, no getty, no session — the
//! machine boots into this process and stays there.
//!
//! Off-image (the dev harness, `tools/dev-run.sh`): the same supervisor and
//! bus without the PID-1 duties, so the mind plane iterates on a normal
//! desktop without rebooting a VM.

mod bus;
mod supervisor;

use std::path::Path;

const BANNER: &str = r#"
   ╭──────────────────────────────────────────────╮
   │                                              │
   │      C L A D E                               │
   │      the living computer                     │
   │                                              │
   │      kernel: hardware only — everything      │
   │      above this line is ours.                │
   │                                              │
   ╰──────────────────────────────────────────────╯
"#;

fn main() -> anyhow::Result<()> {
    let is_pid1 = std::process::id() == 1;

    if is_pid1 {
        mount_pseudo_filesystems();
        // Orphans reparent to us; reap them so nothing zombies.
        spawn_reaper();
    }

    println!("{BANNER}");
    log(
        "weaved",
        &format!("pid {} · pid1={}", std::process::id(), is_pid1),
    );

    // The bus comes up before any service so nobody races it for long.
    let bus = bus::Bus::start()?;

    // Bring up the mind plane in declared order, restart-with-backoff.
    let mut supervisor = supervisor::Supervisor::new(bus.handle());
    for service in clade_proto::SERVICES {
        supervisor.launch(service);
    }

    // M0's definition of ready: every service has announced itself once.
    supervisor.await_all_up();
    bus.handle().broadcast(&clade_proto::Event::WeaveReady)?;
    log(
        "weaved",
        "weave-ready — the Weave would paint here (renderer lands at M2)",
    );

    // PID 1 never exits; the harness runs until interrupted.
    supervisor.run_forever();
}

/// Minimal early-boot mounts. Errors are logged, not fatal: the dev harness
/// hits EPERM here and that is fine, and a missing /proc inside the image is
/// visible in the boot log rather than a silent wedge.
fn mount_pseudo_filesystems() {
    for (source, target, fstype) in [
        ("proc", "/proc", "proc"),
        ("sysfs", "/sys", "sysfs"),
        ("devtmpfs", "/dev", "devtmpfs"),
    ] {
        if Path::new(target).join("self").exists() || Path::new(target).join("null").exists() {
            continue; // already mounted (initramfs did it)
        }
        let _ = std::fs::create_dir_all(target);
        let (src, tgt, fst) = (cstr(source), cstr(target), cstr(fstype));
        let rc = unsafe {
            libc::mount(
                src.as_ptr(),
                tgt.as_ptr(),
                fst.as_ptr(),
                0,
                std::ptr::null(),
            )
        };
        if rc != 0 {
            log(
                "weaved",
                &format!("mount {target} failed: {}", std::io::Error::last_os_error()),
            );
        }
    }
    let _ = std::fs::create_dir_all("/run/clade");
}

/// Reap any child that reparents to PID 1, forever, on a dedicated thread.
/// (The supervisor waits on its own direct children; this catches orphans.)
fn spawn_reaper() {
    std::thread::spawn(|| loop {
        let mut status = 0;
        let pid = unsafe { libc::waitpid(-1, &mut status, 0) };
        if pid <= 0 {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    });
}

fn cstr(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).expect("no interior NULs in mount constants")
}

/// One log line, kernel-console friendly (stdout is the console at PID 1).
pub fn log(who: &str, msg: &str) {
    println!("[clade:{who}] {msg}");
}
