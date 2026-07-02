//! weave — Clade's native userland. M1: boot to a frame.
//!
//! Takes over the display via DRM/KMS (virtio-gpu on the reference machine)
//! and paints Clade's first real graphics: the deep field and the breathing
//! presence dot (docs/07-interaction-model.md). Holding DRM master also keeps
//! the kernel's framebuffer console from drawing over us.
//!
//! M2 grows this into the full Weave shell (zones, Materialize/Dissolve, the
//! renderer decision). Without a DRM device (dev harness, headless CI) it
//! parks quietly so the supervisor doesn't restart-thrash.

use anyhow::{Context, Result};
use drm::buffer::{Buffer as _, DrmFourcc};
use drm::control::{connector, ClipRect, Device as ControlDevice};
use drm::Device;
use std::os::unix::io::AsFd;
use std::time::{Duration, Instant};

/// Palette (matches mockups/shell.css): deep field + teal presence.
const BG_TOP: (u8, u8, u8) = (0x0b, 0x0e, 0x17);
const BG_BOTTOM: (u8, u8, u8) = (0x07, 0x09, 0x0f);
const PRESENCE: (u8, u8, u8) = (0x5e, 0xea, 0xd4);
/// One breath, in seconds (matches the mockup's 3.2s cycle).
const BREATH_SECS: f32 = 3.2;
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
    let (w, h) = (w as u32, h as u32);
    let crtc = *res.crtcs().first().context("no CRTC")?;

    let mut db = card
        .create_dumb_buffer((w, h), DrmFourcc::Xrgb8888, 32)
        .context("create dumb buffer")?;
    let fb = card
        .add_framebuffer(&db, 24, 32)
        .context("add framebuffer")?;
    let pitch = db.pitch() as usize;

    card.set_crtc(crtc, Some(fb), (0, 0), &[con.handle()], Some(mode))
        .context("set CRTC")?;
    log(&format!("display up: {w}x{h} — painting the first frame"));

    // Announce on the bus if it's reachable; keep the connection open so the
    // Weave shows in the lifecycle log. Purely informational in M1.
    let _bus = announce();

    let mut map = card.map_dumb_buffer(&mut db).context("map dumb buffer")?;
    let frame_buf = map.as_mut();

    paint_background(frame_buf, pitch, w, h);
    // Some drivers don't need (or support) dirty rectangles; never fatal.
    card.dirty_framebuffer(fb, &[ClipRect::new(0, 0, w as u16, h as u16)])
        .ok();

    // The presence dot breathes at the mockups' resting spot.
    let (cx, cy) = (w as f32 / 2.0, h as f32 * 0.46);
    let base_r = (h as f32 / 26.0).max(10.0);
    let max_r = base_r * 3.4; // glow reach
    let (x0, y0) = ((cx - max_r).max(0.0) as u32, (cy - max_r).max(0.0) as u32);
    let (x1, y1) = (
        ((cx + max_r) as u32 + 1).min(w),
        ((cy + max_r) as u32 + 1).min(h),
    );

    let start = Instant::now();
    loop {
        let t = start.elapsed().as_secs_f32();
        // 0..1..0 breath, eased.
        let breath = 0.5 - 0.5 * (t * std::f32::consts::TAU / BREATH_SECS).cos();

        paint_region(frame_buf, pitch, h, (x0, y0, x1, y1), |x, y, bg| {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt();

            let core_r = base_r * (0.88 + 0.18 * breath);
            // Solid core with a soft 2px edge.
            let core = smoothstep(core_r + 1.5, core_r - 1.5, d);
            // Wide, faint glow that swells with the breath.
            let glow = smoothstep(max_r, core_r, d).powi(2) * (0.10 + 0.16 * breath);
            let a = (core + glow).min(1.0);
            blend(bg, PRESENCE, a)
        });

        card.dirty_framebuffer(
            fb,
            &[ClipRect::new(x0 as u16, y0 as u16, x1 as u16, y1 as u16)],
        )
        .ok();
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

/// Vertical gradient over the whole frame.
fn paint_background(buf: &mut [u8], pitch: usize, w: u32, h: u32) {
    for y in 0..h {
        let t = y as f32 / h as f32;
        let px = pack(lerp3(BG_TOP, BG_BOTTOM, t));
        let row = &mut buf[y as usize * pitch..y as usize * pitch + w as usize * 4];
        for chunk in row.chunks_exact_mut(4) {
            chunk.copy_from_slice(&px);
        }
    }
}

/// Repaint a rectangular region: background gradient composited by `f`.
fn paint_region(
    buf: &mut [u8],
    pitch: usize,
    h: u32,
    (x0, y0, x1, y1): (u32, u32, u32, u32),
    f: impl Fn(u32, u32, (u8, u8, u8)) -> (u8, u8, u8),
) {
    for y in y0..y1 {
        let bg = lerp3(BG_TOP, BG_BOTTOM, y as f32 / h as f32);
        let row =
            &mut buf[y as usize * pitch + x0 as usize * 4..y as usize * pitch + x1 as usize * 4];
        for (i, chunk) in row.chunks_exact_mut(4).enumerate() {
            chunk.copy_from_slice(&pack(f(x0 + i as u32, y, bg)));
        }
    }
}

/// XRGB8888 little-endian byte order: B, G, R, X.
fn pack((r, g, b): (u8, u8, u8)) -> [u8; 4] {
    [b, g, r, 0]
}

fn lerp3(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    (l(a.0, b.0), l(a.1, b.1), l(a.2, b.2))
}

fn blend(bg: (u8, u8, u8), fg: (u8, u8, u8), a: f32) -> (u8, u8, u8) {
    lerp3(bg, fg, a.clamp(0.0, 1.0))
}

/// 1 at `edge_in`, 0 at `edge_out`, smooth in between.
fn smoothstep(edge_out: f32, edge_in: f32, d: f32) -> f32 {
    let t = ((d - edge_out) / (edge_in - edge_out)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Best-effort lifecycle announcement on the event bus.
fn announce() -> Option<clade_proto::BusClient> {
    let mut bus = clade_proto::BusClient::connect().ok()?;
    bus.publish(&clade_proto::Event::ServiceUp {
        service: "weave".into(),
        pid: std::process::id(),
    })
    .ok()?;
    Some(bus)
}

fn log(msg: &str) {
    println!("[clade:weave] {msg}");
}
