//! Content addressing: blake3 hash → the item's true identity, plus the
//! `sub:item/...` presentation id derived from it.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// `blake3:<64hex>` for a byte slice.
pub fn hash_bytes(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

/// `blake3:<64hex>` for a file, streamed so large files never fully buffer.
pub fn hash_file(path: &Path) -> anyhow::Result<String> {
    let mut hasher = blake3::Hasher::new();
    let mut reader = BufReader::new(File::open(path)?);
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

/// `sub:item/img_<first 12 hex of the content hash>`.
pub fn sub_id_for(content_hash: &str) -> String {
    let hex = content_hash.strip_prefix("blake3:").unwrap_or(content_hash);
    let short: String = hex.chars().take(12).collect();
    format!("sub:item/img_{short}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashid_stable() {
        let h = hash_bytes(b"clade");
        assert!(h.starts_with("blake3:"));
        // blake3 of "clade" is deterministic; assert the derived ids, not the
        // exact digest, so the test documents the shape without pinning a hash
        // we'd have to hand-compute.
        assert_eq!(h.len(), "blake3:".len() + 64);
        let sub = sub_id_for(&h);
        assert!(sub.starts_with("sub:item/img_"));
        assert_eq!(sub.len(), "sub:item/img_".len() + 12);
        // Same bytes → same id.
        assert_eq!(h, hash_bytes(b"clade"));
        // Different bytes → different id.
        assert_ne!(h, hash_bytes(b"clade!"));
    }

    #[test]
    fn sub_id_uses_first_twelve() {
        let sub = sub_id_for("blake3:0123456789abcdeffedcba");
        assert_eq!(sub, "sub:item/img_0123456789ab");
    }
}
