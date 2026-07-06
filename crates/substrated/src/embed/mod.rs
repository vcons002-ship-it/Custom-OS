//! The embedder: image bytes → a normalized feature vector, behind a trait so
//! M4 can swap a CLIP model in with no schema change (the `model` column and
//! `embed_model` meta key key everything on `model_id()`).

mod dct;
mod features;

use crate::config;
use anyhow::{bail, Result};
use image::imageops;
use image::RgbImage;
use std::io::Cursor;

/// A produced embedding plus the EXIF-oriented pixel dimensions.
#[derive(Debug, Clone)]
pub struct Embedding {
    /// L2-normalized; `vec.len() == dim()`.
    pub vec: Vec<f32>,
    pub width: u32,
    pub height: u32,
    /// Detected container format, e.g. "jpeg", for metadata.
    pub format: Option<String>,
}

/// The M4 seam. An embedder maps image bytes to a fixed-dimension vector.
pub trait Embedder: Send + Sync {
    /// Stable id stored in `embeddings.model` and `meta.embed_model`.
    fn model_id(&self) -> &'static str;
    fn dim(&self) -> usize;
    fn embed(&self, image_bytes: &[u8]) -> Result<Embedding>;
}

/// M3's model-free embedder (perceptual DCT + spatial color + HSV histogram).
#[derive(Debug, Default, Clone, Copy)]
pub struct PhashHistEmbedder;

impl PhashHistEmbedder {
    pub fn new() -> Self {
        Self
    }
}

impl Embedder for PhashHistEmbedder {
    fn model_id(&self) -> &'static str {
        crate::db::EMBED_MODEL
    }

    fn dim(&self) -> usize {
        features::DIM
    }

    fn embed(&self, bytes: &[u8]) -> Result<Embedding> {
        if bytes.len() as u64 > config::MAX_BYTES {
            bail!("input exceeds {} byte cap", config::MAX_BYTES);
        }
        // Decode with allocation/dimension limits so a decompression bomb
        // errors instead of OOM-ing the daemon (which runs under panic=abort).
        let mut reader = image::ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|e| anyhow::anyhow!("guess format: {e}"))?;
        let format = reader.format().map(|f| format!("{f:?}").to_lowercase());
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(30_000);
        limits.max_image_height = Some(30_000);
        limits.max_alloc = Some(1024 * 1024 * 1024);
        reader.limits(limits);
        let decoded = reader
            .decode()
            .map_err(|e| anyhow::anyhow!("decode: {e}"))?;

        let (w, h) = (decoded.width(), decoded.height());
        if (w as u64) * (h as u64) > config::MAX_PIXELS {
            bail!("image {}x{} exceeds pixel cap", w, h);
        }

        let exif = crate::exif::read(bytes);
        let oriented = apply_orientation(decoded.to_rgb8(), exif.orientation);
        let (ow, oh) = (oriented.width(), oriented.height());
        let vec = features::featurize(&oriented);
        Ok(Embedding {
            vec,
            width: ow,
            height: oh,
            format,
        })
    }
}

/// Apply an EXIF orientation (1..8) so a rotated duplicate embeds identically.
fn apply_orientation(img: RgbImage, orientation: u8) -> RgbImage {
    match orientation {
        2 => imageops::flip_horizontal(&img),
        3 => imageops::rotate180(&img),
        4 => imageops::flip_vertical(&img),
        5 => imageops::flip_horizontal(&imageops::rotate90(&img)),
        6 => imageops::rotate90(&img),
        7 => imageops::flip_horizontal(&imageops::rotate270(&img)),
        8 => imageops::rotate270(&img),
        _ => img,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};
    use std::io::Cursor;

    fn encode_png(img: &RgbImage) -> Vec<u8> {
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        buf
    }

    fn gradient(w: u32, h: u32, warm: bool) -> RgbImage {
        RgbImage::from_fn(w, h, |x, _y| {
            let t = (x * 255 / w.max(1)) as u8;
            if warm {
                Rgb([220, t / 2, 20])
            } else {
                Rgb([20, t / 2, 220])
            }
        })
    }

    #[test]
    fn embed_deterministic_and_unit_norm() {
        let e = PhashHistEmbedder::new();
        let bytes = encode_png(&gradient(64, 48, true));
        let a = e.embed(&bytes).unwrap();
        let b = e.embed(&bytes).unwrap();
        assert_eq!(a.vec.len(), e.dim());
        assert_eq!(a.vec, b.vec, "same bytes must embed identically");
        let norm = a.vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-4);
    }

    #[test]
    fn embed_rejects_garbage_without_panic() {
        let e = PhashHistEmbedder::new();
        assert!(e.embed(b"not an image at all").is_err());
        // A PNG header followed by garbage.
        let mut junk = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        junk.extend_from_slice(&[0u8; 64]);
        assert!(e.embed(&junk).is_err());
    }

    #[test]
    fn similarity_orders_visually() {
        let e = PhashHistEmbedder::new();
        let red = e.embed(&encode_png(&gradient(64, 48, true))).unwrap();
        // A slightly brighter re-encode of the same warm gradient.
        let red2img = RgbImage::from_fn(64, 48, |x, _| {
            let t = (x * 255 / 64) as u8;
            Rgb([235, t / 2 + 10, 30])
        });
        let red2 = e.embed(&encode_png(&red2img)).unwrap();
        let blue = e.embed(&encode_png(&gradient(64, 48, false))).unwrap();
        let sim = |a: &[f32], b: &[f32]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
        assert!(
            sim(&red.vec, &red2.vec) > sim(&red.vec, &blue.vec),
            "warm↔warm should beat warm↔cool"
        );
    }

    #[test]
    fn resize_invariant() {
        let e = PhashHistEmbedder::new();
        let small = e.embed(&encode_png(&gradient(64, 48, true))).unwrap();
        let big = e.embed(&encode_png(&gradient(128, 96, true))).unwrap();
        let sim: f32 = small.vec.iter().zip(&big.vec).map(|(x, y)| x * y).sum();
        assert!(
            sim > 0.95,
            "2x resize should stay near-identical, got {sim}"
        );
    }

    #[test]
    fn orientation_applied() {
        // We can't easily forge EXIF here; assert the rotation helper instead.
        let img = RgbImage::from_fn(4, 2, |x, y| Rgb([(x * 60) as u8, (y * 120) as u8, 0]));
        let up = apply_orientation(img.clone(), 1);
        let r90 = apply_orientation(img.clone(), 6);
        assert_eq!(up.dimensions(), (4, 2));
        assert_eq!(r90.dimensions(), (2, 4)); // 90° swaps dims
    }
}
