//! Environment resolution and tunable constants for the Substrate.
//!
//! Every path is overridable by env so the dev harness (which has no `/data`)
//! and the hermetic tests can point the daemon at temp directories.

use std::path::PathBuf;

/// SQLite database file. Image default: the persistent data volume.
pub const DB_ENV: &str = "CLADE_SUBSTRATE_DB";
pub const DB_DEFAULT: &str = "/data/substrate.db";

/// Watched library root.
pub const LIBRARY_ENV: &str = "CLADE_LIBRARY";
pub const LIBRARY_DEFAULT: &str = "/data/library";

/// Control socket the CLI talks to when the daemon is up.
pub const CTL_ENV: &str = "CLADE_SUBSTRATE_CTL";
pub const CTL_DEFAULT: &str = "/run/clade/substrated.sock";

/// Live-watch debounce, milliseconds.
pub const DEBOUNCE_ENV: &str = "CLADE_SUBSTRATE_DEBOUNCE_MS";
pub const DEBOUNCE_DEFAULT_MS: u64 = 500;

/// Reject inputs larger than this before decode (decompression-bomb guard).
pub const MAX_BYTES: u64 = 256 * 1024 * 1024;
/// Cap decoded dimensions (a second bomb guard, enforced via image::Limits).
pub const MAX_PIXELS: u64 = 100_000_000;

/// Extensions we treat as candidate images (a fast pre-filter; the real
/// decision is a magic-byte sniff at index time).
pub const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff", "webp"];

pub fn db_path() -> PathBuf {
    env_path(DB_ENV, DB_DEFAULT)
}

pub fn library_path() -> PathBuf {
    env_path(LIBRARY_ENV, LIBRARY_DEFAULT)
}

pub fn ctl_path() -> PathBuf {
    env_path(CTL_ENV, CTL_DEFAULT)
}

pub fn debounce_ms() -> u64 {
    std::env::var(DEBOUNCE_ENV)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEBOUNCE_DEFAULT_MS)
}

fn env_path(key: &str, default: &str) -> PathBuf {
    std::env::var(key)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(default))
}

/// True if `path`'s extension is in the image allowlist (case-insensitive).
pub fn has_image_ext(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .map(|e| IMAGE_EXTS.contains(&e.as_str()))
        .unwrap_or(false)
}
