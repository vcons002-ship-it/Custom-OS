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

// ASCII only: the framebuffer console's default font renders box-drawing
// glyphs as garbage.
const BANNER: &str = r#"
   +----------------------------------------------+
   |                                              |
   |      C L A D E                               |
   |      the living computer                     |
   |                                              |
   |      kernel: hardware only - everything      |
   |      above this line is ours.                |
   |                                              |
   +----------------------------------------------+
"#;

fn main() {
    let is_pid1 = std::process::id() == 1;
    if let Err(e) = run(is_pid1) {
        log("weaved", &format!("fatal: {e}"));
        if is_pid1 {
            // PID 1 must never exit — the kernel panics and the error
            // scrolls away. Hold the machine so the message stays readable.
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        }
        std::process::exit(1);
    }
}

fn run(is_pid1: bool) -> anyhow::Result<()> {
    // Dying takes the whole mind plane with us — proper init behavior in the
    // image, and no orphaned services when the dev harness is Ctrl-C'd/killed.
    install_shutdown_handler();

    if is_pid1 {
        // The kernel gives init an empty-ish environment; anything we or a
        // Capability worker exec later deserves a sane PATH.
        std::env::set_var("PATH", "/usr/sbin:/usr/bin:/sbin:/bin");
        mount_pseudo_filesystems();
        mount_data_volume();
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

    // M0's definition of ready: every service has spawned. Bounded — a
    // missing/crash-looping service must not wedge the machine before
    // weave-ready; we proceed and the supervisor keeps retrying.
    supervisor.await_all_up(std::time::Duration::from_secs(30));
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

/// Mount the persistent data volume (/dev/vdb → /data) if the reference
/// machine attached one (tools/qemu-run.sh always does). Everything that must
/// survive sessions and OS-image rebuilds lives under /data: the Substrate
/// index, Context Graph, Memory, the Current, and the Journal
/// (docs/08-data-knowledge-model.md). Absence is logged, never fatal — the
/// OS still boots; the mind just has nowhere durable to write.
fn mount_data_volume() {
    const DEVICE: &str = "/dev/vdb";
    const TARGET: &str = "/data";
    if !Path::new(DEVICE).exists() {
        log(
            "weaved",
            "no data volume at /dev/vdb — running without durable storage",
        );
        return;
    }
    let _ = std::fs::create_dir_all(TARGET);
    let (src, tgt, fst) = (cstr(DEVICE), cstr(TARGET), cstr("ext4"));
    let rc = unsafe {
        libc::mount(
            src.as_ptr(),
            tgt.as_ptr(),
            fst.as_ptr(),
            0,
            std::ptr::null(),
        )
    };
    if rc == 0 {
        log("weaved", "data volume mounted at /data");
    } else {
        log(
            "weaved",
            &format!(
                "mount {DEVICE} on {TARGET} failed: {}",
                std::io::Error::last_os_error()
            ),
        );
    }
}

/// On SIGTERM/SIGINT: TERM each supervised child precisely (their PIDs live
/// in a signal-safe atomic array — no process-group nuke, which would take
/// the dev harness's parent shell down too), then exit. Async-signal-safe by
/// construction: atomic loads, kill(), _exit() only.
fn install_shutdown_handler() {
    extern "C" fn on_shutdown(_sig: libc::c_int) {
        for slot in &supervisor::CHILD_PIDS {
            let pid = slot.load(std::sync::atomic::Ordering::SeqCst);
            if pid > 0 {
                unsafe { libc::kill(pid, libc::SIGTERM) };
            }
        }
        // PID 1 must never exit — the kernel panics ("attempted to kill
        // init"). Power the machine off properly instead.
        if std::process::id() == 1 {
            unsafe {
                libc::sync();
                libc::reboot(libc::LINUX_REBOOT_CMD_POWER_OFF);
            }
        }
        unsafe { libc::_exit(0) };
    }
    unsafe {
        libc::signal(
            libc::SIGTERM,
            on_shutdown as *const () as libc::sighandler_t,
        );
        libc::signal(libc::SIGINT, on_shutdown as *const () as libc::sighandler_t);
    }
}

/// Reap orphans that reparent to PID 1, forever, on a dedicated thread —
/// WITHOUT stealing the supervisor's children: a blind waitpid(-1) would race
/// the supervisor's Child::wait() and eat its exit statuses. waitid+WNOWAIT
/// peeks without reaping; only non-supervised PIDs are actually collected.
fn spawn_reaper() {
    std::thread::spawn(|| loop {
        unsafe {
            let mut info: libc::siginfo_t = std::mem::zeroed();
            let rc = libc::waitid(libc::P_ALL, 0, &mut info, libc::WEXITED | libc::WNOWAIT);
            if rc != 0 {
                // No children at all right now.
                std::thread::sleep(std::time::Duration::from_millis(200));
                continue;
            }
            let pid = info.si_pid();
            if pid <= 0 {
                std::thread::sleep(std::time::Duration::from_millis(200));
                continue;
            }
            let supervised = supervisor::CHILD_PIDS
                .iter()
                .any(|s| s.load(std::sync::atomic::Ordering::SeqCst) == pid);
            if supervised {
                // The supervisor's Child::wait() will reap it; don't spin on
                // the same zombie in the meantime.
                std::thread::sleep(std::time::Duration::from_millis(100));
            } else {
                libc::waitpid(pid, std::ptr::null_mut(), 0);
            }
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
