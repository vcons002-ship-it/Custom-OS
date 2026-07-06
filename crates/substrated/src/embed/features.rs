//! The 160-dim model-free image feature vector.
//!
//! Blocks (each L2-normalized within-block except aspect, then weighted, then
//! the whole vector is L2-normalized so cosine == dot):
//!   B1 structure  — 63  DCT low-freq coeffs of 32×32 luma, DC dropped (w 1.0)
//!   B2 layout     — 48  4×4 grid of mean YCbCr over 64×64 RGB          (w 1.0)
//!   B3 color hist — 48  HSV Hue×Sat (36) + Value (12), Hellinger       (w 0.7)
//!   B4 aspect     — 1   clamp(log2(w/h),-2,2)/2                         (w 0.3)

use super::dct;
use image::imageops::FilterType;
use image::RgbImage;

const W_STRUCTURE: f32 = 1.0;
const W_LAYOUT: f32 = 1.0;
const W_HIST: f32 = 0.7;
const W_ASPECT: f32 = 0.3;

pub const DIM: usize = 63 + 48 + 48 + 1;

/// Produce the L2-normalized feature vector for an already EXIF-oriented image.
pub fn featurize(img: &RgbImage) -> Vec<f32> {
    let (w, h) = (img.width(), img.height());
    let g32 = image::imageops::resize(img, dct::N as u32, dct::N as u32, FilterType::Triangle);
    let c64 = image::imageops::resize(img, 64, 64, FilterType::Triangle);

    let mut v = Vec::with_capacity(DIM);
    push_block(&mut v, &structure_block(&g32), W_STRUCTURE);
    push_block(&mut v, &layout_block(&c64), W_LAYOUT);
    push_block(&mut v, &hist_block(&c64), W_HIST);
    v.push(aspect_feature(w, h) * W_ASPECT); // scalar, not block-normalized

    debug_assert_eq!(v.len(), DIM);
    l2_normalize(&mut v);
    v
}

/// L2-normalize `block`, scale by `weight`, append to `v`.
fn push_block(v: &mut Vec<f32>, block: &[f32], weight: f32) {
    let mut b = block.to_vec();
    l2_normalize(&mut b);
    for x in b {
        v.push(x * weight);
    }
}

/// B1: 8×8 DCT of 32×32 luma, DC coefficient dropped → 63 values.
fn structure_block(g32: &RgbImage) -> Vec<f32> {
    let mut luma = vec![0f32; dct::N * dct::N];
    for (i, px) in g32.pixels().enumerate() {
        let [r, g, b] = px.0;
        luma[i] = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    }
    let coeffs = dct::dct_top8(&luma);
    coeffs[1..].to_vec() // drop DC → 63
}

/// B2: 4×4 grid of mean (Y, Cb, Cr) over the 64×64 image → 48 values.
fn layout_block(c64: &RgbImage) -> Vec<f32> {
    let mut out = Vec::with_capacity(48);
    for gy in 0..4 {
        for gx in 0..4 {
            let (mut sy, mut scb, mut scr, mut count) = (0f32, 0f32, 0f32, 0f32);
            for y in gy * 16..gy * 16 + 16 {
                for x in gx * 16..gx * 16 + 16 {
                    let [r, g, b] = c64.get_pixel(x, y).0;
                    let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
                    sy += 0.299 * r + 0.587 * g + 0.114 * b;
                    scb += -0.168_736 * r - 0.331_264 * g + 0.5 * b; // centred at 0
                    scr += 0.5 * r - 0.418_688 * g - 0.081_312 * b;
                    count += 1.0;
                }
            }
            out.push(sy / count);
            out.push(scb / count);
            out.push(scr / count);
        }
    }
    out
}

/// B3: HSV histogram — joint Hue(12)×Sat(3)=36 (chromatic pixels only) +
/// Value(12) marginal (all pixels), each sub-hist L1-normalized then √
/// (Hellinger) → 48 values.
fn hist_block(c64: &RgbImage) -> Vec<f32> {
    let mut hs = [0f32; 36];
    let mut val = [0f32; 12];
    let mut hs_count = 0f32;
    let mut val_count = 0f32;
    for px in c64.pixels() {
        let [r, g, b] = px.0;
        let (h, s, v) = rgb_to_hsv(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
        let vb = ((v * 12.0) as usize).min(11);
        val[vb] += 1.0;
        val_count += 1.0;
        if s > 0.12 && v > 0.10 {
            let hb = ((h / 30.0) as usize).min(11);
            let sb = ((s * 3.0) as usize).min(2);
            hs[hb * 3 + sb] += 1.0;
            hs_count += 1.0;
        }
    }
    let mut out = Vec::with_capacity(48);
    for &x in &hs {
        out.push(if hs_count > 0.0 {
            (x / hs_count).sqrt()
        } else {
            0.0
        });
    }
    for &x in &val {
        out.push(if val_count > 0.0 {
            (x / val_count).sqrt()
        } else {
            0.0
        });
    }
    out
}

/// B4: signed log aspect ratio, clamped to [-1, 1].
fn aspect_feature(w: u32, h: u32) -> f32 {
    if w == 0 || h == 0 {
        return 0.0;
    }
    ((w as f32 / h as f32).log2().clamp(-2.0, 2.0)) / 2.0
}

/// RGB (0..1) → HSV with H in [0,360), S,V in [0,1].
fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let v = max;
    let s = if max <= 0.0 { 0.0 } else { delta / max };
    let h = if delta <= 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta).rem_euclid(6.0))
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    (h.rem_euclid(360.0), s, v)
}

fn l2_normalize(v: &mut [f32]) {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-12 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    fn solid(w: u32, h: u32, c: [u8; 3]) -> RgbImage {
        RgbImage::from_pixel(w, h, Rgb(c))
    }

    #[test]
    fn output_is_unit_norm_and_dim() {
        let v = featurize(&solid(80, 60, [200, 30, 30]));
        assert_eq!(v.len(), DIM);
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-4, "norm={norm}");
    }

    #[test]
    fn hsv_primaries() {
        let (h, s, v) = rgb_to_hsv(1.0, 0.0, 0.0);
        assert!(h.abs() < 1.0 && (s - 1.0).abs() < 1e-6 && (v - 1.0).abs() < 1e-6);
        let (h, _, _) = rgb_to_hsv(0.0, 1.0, 0.0);
        assert!((h - 120.0).abs() < 1.0);
        let (h, _, _) = rgb_to_hsv(0.0, 0.0, 1.0);
        assert!((h - 240.0).abs() < 1.0);
    }

    #[test]
    fn aspect_sign() {
        assert!(aspect_feature(200, 100) > 0.0); // landscape
        assert!(aspect_feature(100, 200) < 0.0); // portrait
        assert!(aspect_feature(100, 100).abs() < 1e-6); // square
    }
}
