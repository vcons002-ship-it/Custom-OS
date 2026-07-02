//! The Weave's software compositor primitives: a mapped XRGB8888 frame plus
//! the draw kit the shell is built from (renderer decision: grow this — see
//! docs/phase-1-plan.md).

pub type Rgb = (u8, u8, u8);

pub const BG_TOP: Rgb = (0x0b, 0x0e, 0x17);
pub const BG_BOTTOM: Rgb = (0x07, 0x09, 0x0f);
pub const INK: Rgb = (0xe8, 0xec, 0xf4);
pub const INK_DIM: Rgb = (0x9a, 0xa3, 0xb5);
pub const INK_FAINT: Rgb = (0x5b, 0x64, 0x78);
pub const PRESENCE: Rgb = (0x5e, 0xea, 0xd4);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
    pub fn inflate(self, m: i32) -> Rect {
        Rect::new(self.x - m, self.y - m, self.w + 2 * m, self.h + 2 * m)
    }
}

/// A frame we can draw into: the mapped dumb buffer.
pub struct Frame<'a> {
    pub buf: &'a mut [u8],
    pub pitch: usize,
    pub w: i32,
    pub h: i32,
}

impl Frame<'_> {
    pub fn bg_at(&self, y: i32) -> Rgb {
        lerp3(BG_TOP, BG_BOTTOM, y as f32 / self.h as f32)
    }

    fn put(&mut self, x: i32, y: i32, c: Rgb) {
        let i = y as usize * self.pitch + x as usize * 4;
        self.buf[i] = c.2;
        self.buf[i + 1] = c.1;
        self.buf[i + 2] = c.0;
        self.buf[i + 3] = 0;
    }

    fn get(&self, x: i32, y: i32) -> Rgb {
        let i = y as usize * self.pitch + x as usize * 4;
        (self.buf[i + 2], self.buf[i + 1], self.buf[i])
    }

    /// Blend `c` over the current pixel with coverage `a` (0..1).
    pub fn blend(&mut self, x: i32, y: i32, c: Rgb, a: f32) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h || a <= 0.0 {
            return;
        }
        let base = self.get(x, y);
        self.put(x, y, lerp3(base, c, a.min(1.0)));
    }

    /// Repaint the background gradient over a region.
    pub fn clear_region(&mut self, r: Rect) {
        let x0 = r.x.max(0);
        let x1 = (r.x + r.w).min(self.w);
        let y0 = r.y.max(0);
        let y1 = (r.y + r.h).min(self.h);
        for y in y0..y1 {
            let px = pack(self.bg_at(y));
            let row = &mut self.buf[y as usize * self.pitch + x0 as usize * 4
                ..y as usize * self.pitch + x1 as usize * 4];
            for chunk in row.chunks_exact_mut(4) {
                chunk.copy_from_slice(&px);
            }
        }
    }

    /// A translucent "glass" panel: brightens the field, rounded corners,
    /// hairline border — the card/chip look from the mockups.
    pub fn glass(&mut self, r: Rect, radius: f32, fill_a: f32, border_a: f32) {
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                let cov = rounded_coverage(r, radius, x, y);
                if cov <= 0.0 {
                    continue;
                }
                let edge = rounded_edge(r, radius, x, y);
                self.blend(x, y, (0xff, 0xff, 0xff), fill_a * cov);
                if edge > 0.0 {
                    self.blend(x, y, (0xff, 0xff, 0xff), border_a * edge * cov);
                }
            }
        }
    }

    /// A filled disc with a soft edge (the presence dot core).
    pub fn disc(&mut self, cx: f32, cy: f32, radius: f32, c: Rgb, alpha: f32) {
        let r = radius + 2.0;
        let (x0, y0) = ((cx - r) as i32, (cy - r) as i32);
        let (x1, y1) = ((cx + r) as i32 + 1, (cy + r) as i32 + 1);
        for y in y0..y1 {
            for x in x0..x1 {
                let d = dist(x, y, cx, cy);
                let cov = (radius + 0.5 - d).clamp(0.0, 1.0);
                self.blend(x, y, c, alpha * cov);
            }
        }
    }

    /// A soft radial glow (quadratic falloff) around a point.
    pub fn glow(&mut self, cx: f32, cy: f32, inner: f32, outer: f32, c: Rgb, alpha: f32) {
        let (x0, y0) = ((cx - outer) as i32, (cy - outer) as i32);
        let (x1, y1) = ((cx + outer) as i32 + 1, (cy + outer) as i32 + 1);
        for y in y0..y1 {
            for x in x0..x1 {
                let d = dist(x, y, cx, cy);
                if d >= outer {
                    continue;
                }
                let t = ((outer - d) / (outer - inner)).clamp(0.0, 1.0);
                self.blend(x, y, c, alpha * t * t);
            }
        }
    }
}

fn dist(x: i32, y: i32, cx: f32, cy: f32) -> f32 {
    let dx = x as f32 + 0.5 - cx;
    let dy = y as f32 + 0.5 - cy;
    (dx * dx + dy * dy).sqrt()
}

/// Antialiased coverage of a rounded rectangle at pixel (x, y).
fn rounded_coverage(r: Rect, radius: f32, x: i32, y: i32) -> f32 {
    let fx = x as f32 + 0.5;
    let fy = y as f32 + 0.5;
    // Signed distance to the rounded rect.
    let cx = fx.clamp(r.x as f32 + radius, (r.x + r.w) as f32 - radius);
    let cy = fy.clamp(r.y as f32 + radius, (r.y + r.h) as f32 - radius);
    let dx = fx - cx;
    let dy = fy - cy;
    let d = (dx * dx + dy * dy).sqrt();
    (radius + 0.5 - d).clamp(0.0, 1.0)
}

/// 1.0 on the 1px inner border ring of the rounded rect, 0 elsewhere.
fn rounded_edge(r: Rect, radius: f32, x: i32, y: i32) -> f32 {
    let fx = x as f32 + 0.5;
    let fy = y as f32 + 0.5;
    let cx = fx.clamp(r.x as f32 + radius, (r.x + r.w) as f32 - radius);
    let cy = fy.clamp(r.y as f32 + radius, (r.y + r.h) as f32 - radius);
    let dx = fx - cx;
    let dy = fy - cy;
    let d = (dx * dx + dy * dy).sqrt();
    (1.0 - (radius - 1.0 - d).abs()).clamp(0.0, 1.0)
}

pub fn pack((r, g, b): Rgb) -> [u8; 4] {
    [b, g, r, 0]
}

pub fn lerp3(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    (l(a.0, b.0), l(a.1, b.1), l(a.2, b.2))
}
