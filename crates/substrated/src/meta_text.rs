//! Assemble the FTS5 body for an item from its path and camera metadata.
//! M4 adds captions/OCR here; for M3 it is filename + path words + camera.

use std::path::Path;

/// The `filename` FTS column: the basename with separators turned to spaces.
pub fn filename_field(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(tokenize)
        .unwrap_or_default()
}

/// The `text` FTS column: parent path words + camera make/model.
pub fn text_field(path: &Path, make: Option<&str>, model: Option<&str>) -> String {
    let mut parts = Vec::new();
    if let Some(parent) = path.parent().and_then(|p| p.to_str()) {
        parts.push(tokenize(parent));
    }
    if let Some(m) = make {
        parts.push(m.to_string());
    }
    if let Some(m) = model {
        parts.push(m.to_string());
    }
    parts
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Split on non-alphanumeric so "beach_sunset.jpg" → "beach sunset jpg".
fn tokenize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_filename() {
        assert_eq!(
            filename_field(Path::new("/a/beach_sunset.jpg")),
            "beach sunset jpg"
        );
    }

    #[test]
    fn text_includes_camera() {
        let t = text_field(
            Path::new("/photos/trip/x.jpg"),
            Some("Canon"),
            Some("EOS R5"),
        );
        assert!(t.contains("photos"));
        assert!(t.contains("trip"));
        assert!(t.contains("Canon"));
        assert!(t.contains("EOS"));
    }
}
