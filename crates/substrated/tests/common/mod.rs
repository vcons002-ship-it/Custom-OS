//! Shared helpers for substrated integration tests: synthetic image clusters
//! written to a temp library.
//!
//! Each test binary includes this module and uses a different subset, so some
//! helpers look unused per-binary.
#![allow(dead_code)]

use image::{Rgb, RgbImage};
use std::path::Path;

/// Four visually distinct clusters; `variant` nudges pixels so members are
/// near-duplicates, not identical (distinct content hashes).
pub fn cluster_image(cluster: usize, variant: u32) -> RgbImage {
    let v = (variant % 20) as i32;
    RgbImage::from_fn(96, 72, |x, y| match cluster {
        // Warm / red gradient.
        0 => Rgb([(200 + v) as u8, (x * 60 / 96) as u8, 20]),
        // Cool / blue gradient.
        1 => Rgb([20, (y * 60 / 72) as u8, (200 + v) as u8]),
        // Green checkerboard.
        2 => {
            if ((x / 8) + (y / 8)) % 2 == 0 {
                Rgb([20, (200 + v) as u8, 40])
            } else {
                Rgb([10, 80, 20])
            }
        }
        // Gray noise-ish.
        _ => {
            let n = (x as usize * 7 + y as usize * 13 + variant as usize) % 60;
            let g = (n as i32 + 100 + v).clamp(0, 255) as u8;
            Rgb([g, g, g])
        }
    })
}

/// Write `img` as a PNG at `path`.
pub fn write_png(img: &RgbImage, path: &Path) {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).unwrap();
    }
    img.save_with_format(path, image::ImageFormat::Png).unwrap();
}

/// Populate `dir` with `per_cluster` images in each of 4 clusters. Returns the
/// written paths grouped by cluster.
pub fn populate_library(dir: &Path, per_cluster: u32) -> Vec<Vec<std::path::PathBuf>> {
    let mut groups = vec![Vec::new(); 4];
    for (cluster, group) in groups.iter_mut().enumerate() {
        for variant in 0..per_cluster {
            let p = dir.join(format!("c{cluster}_v{variant}.png"));
            write_png(&cluster_image(cluster, variant), &p);
            group.push(p);
        }
    }
    groups
}
