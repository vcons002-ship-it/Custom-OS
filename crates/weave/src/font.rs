//! Text for the Weave: DejaVu Sans (vendored, see assets/fonts/), rasterized
//! by fontdue with a small per-size glyph cache.

use crate::fb::{Frame, Rgb};
use std::collections::HashMap;

const FONT_BYTES: &[u8] = include_bytes!("../../../assets/fonts/DejaVuSans.ttf");

pub struct Text {
    font: fontdue::Font,
    cache: HashMap<(char, u32), (fontdue::Metrics, Vec<u8>)>,
}

impl Text {
    pub fn new() -> Self {
        let font = fontdue::Font::from_bytes(FONT_BYTES, fontdue::FontSettings::default())
            .expect("embedded DejaVuSans parses");
        Self {
            font,
            cache: HashMap::new(),
        }
    }

    fn glyph(&mut self, ch: char, px: f32) -> &(fontdue::Metrics, Vec<u8>) {
        let key = (ch, px as u32);
        if !self.cache.contains_key(&key) {
            let g = self.font.rasterize(ch, px);
            self.cache.insert(key, g);
        }
        &self.cache[&key]
    }

    pub fn width(&mut self, s: &str, px: f32) -> i32 {
        s.chars()
            .map(|c| self.glyph(c, px).0.advance_width)
            .sum::<f32>() as i32
    }

    /// Draw `s` with its baseline anchored so the text's TOP is at `y`.
    /// Returns the advance width drawn.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        f: &mut Frame,
        s: &str,
        x: i32,
        y: i32,
        px: f32,
        c: Rgb,
        alpha: f32,
    ) -> i32 {
        let ascent = self
            .font
            .horizontal_line_metrics(px)
            .map(|m| m.ascent)
            .unwrap_or(px * 0.8);
        let baseline = y as f32 + ascent;
        let mut pen = x as f32;
        for ch in s.chars() {
            let (metrics, coverage) = {
                let g = self.glyph(ch, px);
                (g.0, g.1.clone())
            };
            let gx = pen as i32 + metrics.xmin;
            let gy = (baseline - metrics.height as f32 - metrics.ymin as f32) as i32;
            for (i, cov) in coverage.iter().enumerate() {
                if *cov == 0 {
                    continue;
                }
                let px_x = gx + (i % metrics.width) as i32;
                let px_y = gy + (i / metrics.width) as i32;
                f.blend(px_x, px_y, c, alpha * (*cov as f32 / 255.0));
            }
            pen += metrics.advance_width;
        }
        (pen - x as f32) as i32
    }
}
