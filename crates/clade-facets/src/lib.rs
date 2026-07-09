//! Facet Resolution — "what *is* this content?" (docs/04-ai-architecture.md §2).
//!
//! Facets replace rigid file types: a `.txt` may be *notes*, a *to-do list*, or
//! *code*; a screenshot of code is both *image* and *code*. Resolution is
//! additive and confidence-scored. This crate is the cheap deterministic tier
//! (magic bytes + extension + text heuristics); the model tier plugs in via
//! [`FacetRefiner`] (M4, on the owner's GPU) with no change to the output shape.

mod ext;
mod magic;
mod text;

pub use ext::code_from_ext;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// One semantic interpretation of content, with confidence in [0, 1].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Facet {
    /// e.g. `image`, `pdf`, `document`, `code`, `text`, `notes`, `todo`,
    /// `markdown`, `data`, `archive`, `binary`.
    pub kind: String,
    pub confidence: f32,
    /// Sub-classification: image format, code language, data flavour, …
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub detail: Option<String>,
}

impl Facet {
    pub fn new(kind: &str, confidence: f32, detail: Option<&str>) -> Self {
        Self {
            kind: kind.into(),
            confidence,
            detail: detail.map(str::to_string),
        }
    }
}

/// Resolve the facets of `bytes` (a leading slice is enough), using `name`
/// (a filename or path) for the extension hint. Results are additive, merged
/// by kind (highest confidence wins), and sorted most-confident first.
pub fn resolve(name: &str, bytes: &[u8]) -> Vec<Facet> {
    let mut facets: Vec<Facet> = Vec::new();

    let magic_facet = magic::sniff(bytes);
    // A recognized binary container (image/pdf/archive/executable) is never
    // "text", even if a tiny header sample happens to be ASCII-clean.
    let binary_container = magic_facet
        .as_ref()
        .map(|f| matches!(f.kind.as_str(), "image" | "pdf" | "archive" | "binary"))
        .unwrap_or(false);
    if let Some(f) = magic_facet {
        facets.push(f);
    }
    let ext = extension(name);
    facets.extend(ext::from_extension(&ext));

    if !binary_container && text::is_text(bytes) {
        let head = utf8_head(bytes);
        facets.extend(text::text_facets(&ext, &head));
    }

    merge(facets)
}

/// Resolve directly from a filesystem path (reads a 64 KiB head).
pub fn resolve_path(path: &Path) -> std::io::Result<Vec<Facet>> {
    use std::io::Read;
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let mut buf = vec![0u8; 64 * 1024];
    let n = std::fs::File::open(path)?.read(&mut buf)?;
    buf.truncate(n);
    Ok(resolve(name, &buf))
}

/// The model tier: refine/augment deterministic facets (e.g. distinguish an
/// *invoice* PDF from a *contract*, or classify an ambiguous image). M4's
/// on-device model implements this; [`NullRefiner`] is the pre-model default.
pub trait FacetRefiner: Send + Sync {
    fn refine(&self, name: &str, bytes: &[u8], facets: &mut Vec<Facet>);
}

/// No-op refiner used before the model tier exists.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullRefiner;
impl FacetRefiner for NullRefiner {
    fn refine(&self, _name: &str, _bytes: &[u8], _facets: &mut Vec<Facet>) {}
}

/// Deterministic resolution followed by a model refinement pass.
pub fn resolve_with(name: &str, bytes: &[u8], refiner: &dyn FacetRefiner) -> Vec<Facet> {
    let mut facets = resolve(name, bytes);
    refiner.refine(name, bytes, &mut facets);
    facets.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    facets
}

/// The single highest-confidence facet kind (the "primary" mode), if any.
pub fn primary(facets: &[Facet]) -> Option<&Facet> {
    facets.iter().max_by(|a, b| {
        a.confidence
            .partial_cmp(&b.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn extension(name: &str) -> String {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default()
}

fn utf8_head(bytes: &[u8]) -> String {
    let sample = &bytes[..bytes.len().min(8192)];
    match std::str::from_utf8(sample) {
        Ok(s) => s.to_string(),
        Err(e) => String::from_utf8_lossy(&sample[..e.valid_up_to()]).into_owned(),
    }
}

/// Merge duplicate kinds (keep the highest confidence and any detail), drop the
/// bare `text` facet when a more specific text kind is present, fall back to
/// `binary`, and sort most-confident first.
fn merge(facets: Vec<Facet>) -> Vec<Facet> {
    let mut by_kind: HashMap<String, Facet> = HashMap::new();
    for f in facets {
        by_kind
            .entry(f.kind.clone())
            .and_modify(|e| {
                if f.confidence > e.confidence {
                    e.confidence = f.confidence;
                }
                if e.detail.is_none() {
                    e.detail = f.detail.clone();
                }
            })
            .or_insert(f);
    }
    let mut out: Vec<Facet> = by_kind.into_values().collect();
    if out.is_empty() {
        out.push(Facet::new("binary", 0.5, None));
    }
    out.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.kind.cmp(&b.kind))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(fs: &[Facet]) -> Vec<&str> {
        fs.iter().map(|f| f.kind.as_str()).collect()
    }
    fn detail_of<'a>(fs: &'a [Facet], kind: &str) -> Option<&'a str> {
        fs.iter()
            .find(|f| f.kind == kind)
            .and_then(|f| f.detail.as_deref())
    }

    #[test]
    fn jpeg_by_magic_even_with_wrong_extension() {
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0, 0, 0, 0, 0];
        let f = resolve("photo.txt", &jpeg);
        assert_eq!(primary(&f).unwrap().kind, "image");
        assert_eq!(detail_of(&f, "image"), Some("jpeg"));
        assert!(
            primary(&f).unwrap().confidence > 0.9,
            "magic beats a lying extension"
        );
    }

    #[test]
    fn png_magic() {
        let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 1, 2, 3];
        assert_eq!(primary(&resolve("x.png", &png)).unwrap().kind, "image");
    }

    #[test]
    fn pdf_magic() {
        let f = resolve("report.pdf", b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n");
        assert_eq!(primary(&f).unwrap().kind, "pdf");
    }

    #[test]
    fn rust_code_by_extension() {
        let src = b"fn main() {\n    println!(\"hi\");\n}\n";
        let f = resolve("main.rs", src);
        assert!(kinds(&f).contains(&"code"));
        assert_eq!(detail_of(&f, "code"), Some("rust"));
    }

    #[test]
    fn shell_script_by_shebang_no_extension() {
        let src = b"#!/usr/bin/env bash\nset -e\necho hi\n";
        let f = resolve("deploy", src);
        assert!(kinds(&f).contains(&"code"));
        assert_eq!(detail_of(&f, "code"), Some("shell"));
    }

    #[test]
    fn todo_list_is_todo_and_text() {
        let src = b"groceries\n- [ ] milk\n- [x] eggs\n- [ ] bread\n";
        let f = resolve("list.txt", src);
        assert!(kinds(&f).contains(&"todo"));
        assert!(kinds(&f).contains(&"text"));
    }

    #[test]
    fn markdown_detected() {
        let src = b"# Title\n\nSome intro.\n\n- one\n- two\n\nSee [link](http://x).\n";
        let f = resolve("notes.md", src);
        assert!(kinds(&f).contains(&"markdown"));
    }

    #[test]
    fn json_data() {
        let src = br#"{"name":"clade","version":1}"#;
        let f = resolve("pkg.json", src);
        assert_eq!(detail_of(&f, "data"), Some("json"));
    }

    #[test]
    fn csv_by_content_without_extension() {
        let src = b"name,age,city\nalice,30,NYC\nbob,25,LA\n";
        let f = resolve("people", src);
        assert!(kinds(&f).contains(&"data"));
    }

    #[test]
    fn svg_is_image_and_code() {
        let src = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><rect/></svg>";
        let f = resolve("icon.svg", src);
        assert!(kinds(&f).contains(&"image"));
        assert!(kinds(&f).contains(&"code"));
    }

    #[test]
    fn prose_is_notes() {
        let src = b"The trip was lovely. We should book the cottage again in September.";
        let f = resolve("memo.txt", src);
        assert!(kinds(&f).contains(&"text"));
        assert!(kinds(&f).contains(&"notes"));
    }

    #[test]
    fn unknown_binary_falls_back() {
        let f = resolve("blob.dat", &[0u8, 1, 2, 3, 255, 254]);
        assert_eq!(primary(&f).unwrap().kind, "binary");
    }

    #[test]
    fn facet_json_shape_is_stable() {
        let f = Facet::new("image", 0.98, Some("jpeg"));
        assert_eq!(
            serde_json::to_string(&f).unwrap(),
            r#"{"kind":"image","confidence":0.98,"detail":"jpeg"}"#
        );
        let bare = Facet::new("text", 0.6, None);
        assert_eq!(
            serde_json::to_string(&bare).unwrap(),
            r#"{"kind":"text","confidence":0.6}"#
        );
    }

    #[test]
    fn refiner_seam_runs() {
        struct Tagger;
        impl FacetRefiner for Tagger {
            fn refine(&self, _n: &str, _b: &[u8], facets: &mut Vec<Facet>) {
                if facets.iter().any(|f| f.kind == "pdf") {
                    facets.push(Facet::new("invoice", 0.7, None));
                }
            }
        }
        let f = resolve_with("bill.pdf", b"%PDF-1.4", &Tagger);
        assert!(f.iter().any(|x| x.kind == "invoice"));
    }
}
