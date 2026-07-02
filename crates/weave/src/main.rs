//! weave — Clade's native userland. M2: the Weave shell.
//!
//! Takes over the display via DRM/KMS (M1) and now composes the idle Weave
//! from the mockups: greeting on the Stage, a live rail fed by the event
//! bus, the Intent Bar with real keyboard input (raw evdev), and the
//! presence dot breathing beside it. Renderer decision (see
//! docs/phase-1-plan.md): this software compositor — zero dependencies,
//! 30fps damage-driven repaints — grows with the OS; GPU rendering returns
//! when animation complexity demands it.
//!
//! Without a DRM device (dev harness, headless CI) it parks quietly so the
//! supervisor doesn't restart-thrash.

mod fb;
mod font;
mod input;
mod scene;

use anyhow::{Context, Result};
use drm::buffer::{Buffer as _, DrmFourcc};
use drm::control::{connector, ClipRect, Device as ControlDevice};
use drm::Device;
use fb::Frame;
use std::os::unix::io::AsFd;
use std::sync::mpsc;
use std::time::Duration;

const FRAME: Duration = Duration::from_millis(33); // ~30 fps

struct Card(std::fs::File);

impl AsFd for Card {
    fn as_fd(&self) -> std::os::unix::io::BorrowedFd<'_> {
        self.0.as_fd()
    }
}
impl Device for Card {}
impl ControlDevice for Card {}

fn main() {
    if let Err(e) = run() {
        log(&format!("{e:#}"));
        log("no display to paint — parking (the supervisor keeps me alive)");
        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    }
}

fn run() -> Result<()> {
    let card = open_card().context("no DRM device (/dev/dri/card*)")?;

    // Master keeps fbcon and any other client from touching the display.
    if card.acquire_master_lock().is_err() {
        log("could not acquire DRM master (continuing — likely already master)");
    }

    let res = card.resource_handles().context("resource handles")?;
    let con = res
        .connectors()
        .iter()
        .filter_map(|h| card.get_connector(*h, true).ok())
        .find(|c| c.state() == connector::State::Connected)
        .context("no connected display connector")?;
    let &mode = con.modes().first().context("connector has no modes")?;
    let (w, h) = mode.size();
    let (w, h) = (w as i32, h as i32);
    let crtc = *res.crtcs().first().context("no CRTC")?;

    let mut db = card
        .create_dumb_buffer((w as u32, h as u32), DrmFourcc::Xrgb8888, 32)
        .context("create dumb buffer")?;
    let fbh = card
        .add_framebuffer(&db, 24, 32)
        .context("add framebuffer")?;
    let pitch = db.pitch() as usize;

    card.set_crtc(crtc, Some(fbh), (0, 0), &[con.handle()], Some(mode))
        .context("set CRTC")?;
    log(&format!("display up: {w}x{h} — composing the Weave"));

    // Keyboard(s), straight from evdev.
    let (key_tx, key_rx) = mpsc::channel();
    let keyboards = input::spawn_keyboards(key_tx);
    log(&format!("keyboards: {keyboards}"));

    // Bus: announce, then feed service liveness into the scene.
    let (bus_tx, bus_rx) = mpsc::channel::<String>();
    std::thread::spawn(move || bus_listener(bus_tx));

    let mut map = card.map_dumb_buffer(&mut db).context("map dumb buffer")?;
    let mut frame = Frame {
        buf: map.as_mut(),
        pitch,
        w,
        h,
    };

    // First paint: the whole field.
    frame.clear_region(fb::Rect::new(0, 0, w, h));
    card.dirty_framebuffer(fbh, &[ClipRect::new(0, 0, w as u16, h as u16)])
        .ok();

    let mut text = font::Text::new();
    let mut scene = scene::Scene::new(w, h);

    loop {
        let mut damage: Vec<scene::Element> = scene.tick();

        for name in bus_rx.try_iter() {
            scene.note_service(name);
        }
        for ev in key_rx.try_iter() {
            match ev {
                input::KeyEvent::Char(c) => damage.push(scene.key_char(c)),
                input::KeyEvent::Backspace => damage.push(scene.key_backspace()),
                input::KeyEvent::Enter => {
                    let (said, touched) = scene.key_enter();
                    if !said.is_empty() {
                        log(&format!("intent: {said}"));
                    }
                    damage.extend(touched);
                }
            }
        }

        damage.sort_by_key(|e| *e as u8);
        damage.dedup();
        let mut clips = Vec::new();
        for e in damage {
            scene.draw(&mut frame, &mut text, e);
            let r = scene.rect_of(e);
            clips.push(ClipRect::new(
                r.x.max(0) as u16,
                r.y.max(0) as u16,
                (r.x + r.w).clamp(0, w) as u16,
                (r.y + r.h).clamp(0, h) as u16,
            ));
        }
        if !clips.is_empty() {
            card.dirty_framebuffer(fbh, &clips).ok();
        }
        std::thread::sleep(FRAME);
    }
}

fn open_card() -> Result<Card> {
    for n in 0..4 {
        let path = format!("/dev/dri/card{n}");
        if let Ok(f) = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
        {
            log(&format!("using {path}"));
            return Ok(Card(f));
        }
    }
    anyhow::bail!("no /dev/dri/card0..3")
}

/// Announce the Weave, then relay every service seen on the bus (ServiceUp
/// and heartbeats alike) to the scene for the liveness card.
fn bus_listener(tx: mpsc::Sender<String>) {
    let Ok(mut bus) = clade_proto::BusClient::connect() else {
        log("bus unreachable — rail shows no liveness");
        return;
    };
    let _ = bus.publish(&clade_proto::Event::ServiceUp {
        service: "weave".into(),
        pid: std::process::id(),
    });
    loop {
        match bus.next_event() {
            Ok(clade_proto::Event::ServiceUp { service, .. })
            | Ok(clade_proto::Event::Heartbeat { service, .. }) => {
                if tx.send(service).is_err() {
                    return;
                }
            }
            Ok(_) => {}
            Err(_) => return,
        }
    }
}

fn log(msg: &str) {
    println!("[clade:weave] {msg}");
}
