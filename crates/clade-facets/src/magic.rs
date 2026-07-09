//! Magic-byte sniffing — the strongest, cheapest signal. A container format
//! is nearly certain from its header, so these carry high confidence.

use crate::Facet;

/// Identify a facet from leading bytes, if a known signature matches.
pub fn sniff(bytes: &[u8]) -> Option<Facet> {
    let b = bytes;
    let starts = |sig: &[u8]| b.len() >= sig.len() && &b[..sig.len()] == sig;

    // Images.
    if starts(&[0xFF, 0xD8, 0xFF]) {
        return Some(img(0.99, "jpeg"));
    }
    if starts(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some(img(0.99, "png"));
    }
    if starts(b"GIF87a") || starts(b"GIF89a") {
        return Some(img(0.99, "gif"));
    }
    if starts(b"BM") {
        return Some(img(0.9, "bmp"));
    }
    if starts(&[0x49, 0x49, 0x2A, 0x00]) || starts(&[0x4D, 0x4D, 0x00, 0x2A]) {
        return Some(img(0.95, "tiff"));
    }
    if b.len() >= 12 && &b[..4] == b"RIFF" && &b[8..12] == b"WEBP" {
        return Some(img(0.99, "webp"));
    }
    // HEIC/HEIF: ftyp box with heic/heif/mif1 brand.
    if b.len() >= 12 && &b[4..8] == b"ftyp" {
        let brand = &b[8..12];
        if matches!(brand, b"heic" | b"heix" | b"heif" | b"mif1" | b"hevc") {
            return Some(img(0.95, "heic"));
        }
    }

    // Documents / other containers.
    if starts(b"%PDF-") {
        return Some(facet("pdf", 0.99, Some("pdf")));
    }
    if starts(&[0x1F, 0x8B]) {
        return Some(facet("archive", 0.95, Some("gzip")));
    }
    if starts(&[b'P', b'K', 0x03, 0x04]) {
        // Could be a plain zip or an OOXML/ODF document; caller's extension
        // refines it. Report the archive container here.
        return Some(facet("archive", 0.8, Some("zip")));
    }
    if starts(&[0x7F, b'E', b'L', b'F']) {
        return Some(facet("binary", 0.98, Some("elf")));
    }

    None
}

fn img(conf: f32, detail: &str) -> Facet {
    facet("image", conf, Some(detail))
}

fn facet(kind: &str, conf: f32, detail: Option<&str>) -> Facet {
    Facet {
        kind: kind.into(),
        confidence: conf,
        detail: detail.map(str::to_string),
    }
}
