//! Vector storage (f32 ⇄ little-endian BLOB) and brute-force top-N cosine
//! search. Vectors are stored L2-normalized, so cosine similarity is a plain
//! dot product. Brute force is the right call at personal scale (hundreds–low
//! thousands of images); sqlite-vec is deliberately avoided (see docs).

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Serialize a vector as little-endian f32 bytes.
pub fn to_blob(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for &x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out
}

/// Parse a little-endian f32 BLOB. Returns `None` for a malformed length so a
/// corrupt row is skipped rather than crashing the search.
pub fn from_blob(b: &[u8]) -> Option<Vec<f32>> {
    if b.is_empty() || !b.len().is_multiple_of(4) {
        return None;
    }
    Some(
        b.chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
    )
}

/// Dot product; for L2-normalized inputs this is cosine similarity.
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// One candidate in the search cache.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub item_id: i64,
    pub content_hash: String,
    pub vec: Vec<f32>,
}

/// A scored neighbour.
#[derive(Debug, Clone, PartialEq)]
pub struct Neighbor {
    pub item_id: i64,
    pub score: f32,
}

// Min-ordering wrapper so a BinaryHeap (a max-heap) can keep the *smallest*
// score at the top, letting us bound the heap to N and pop the weakest.
struct MinScored(Neighbor);
impl PartialEq for MinScored {
    fn eq(&self, o: &Self) -> bool {
        self.0.score == o.0.score
    }
}
impl Eq for MinScored {}
impl PartialOrd for MinScored {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for MinScored {
    fn cmp(&self, o: &Self) -> Ordering {
        // Reverse so the heap root is the lowest score. NaN sorts as lowest.
        o.0.score
            .partial_cmp(&self.0.score)
            .unwrap_or(Ordering::Equal)
    }
}

/// Top-N neighbours of `query` among `entries`, excluding any entry whose
/// content hash equals `exclude_hash` (the query image itself). Results are
/// sorted by descending score. `dim`-mismatched entries are skipped.
pub fn top_n(query: &[f32], entries: &[CacheEntry], exclude_hash: &str, n: usize) -> Vec<Neighbor> {
    if n == 0 {
        return Vec::new();
    }
    // Bound the pre-allocation to the data size — `n` comes from `--top` and a
    // huge value would otherwise overflow/abort the allocation.
    let mut heap: BinaryHeap<MinScored> =
        BinaryHeap::with_capacity(n.min(entries.len()).saturating_add(1));
    for e in entries {
        if e.content_hash == exclude_hash || e.vec.len() != query.len() {
            continue;
        }
        let score = dot(query, &e.vec);
        heap.push(MinScored(Neighbor {
            item_id: e.item_id,
            score,
        }));
        if heap.len() > n {
            heap.pop(); // drop the current weakest
        }
    }
    let mut out: Vec<Neighbor> = heap.into_iter().map(|m| m.0).collect();
    out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_blob_roundtrip() {
        let v = vec![0.0f32, 1.5, -2.25, 3.75];
        let blob = to_blob(&v);
        assert_eq!(blob.len(), 16);
        assert_eq!(from_blob(&blob).unwrap(), v);
    }

    #[test]
    fn malformed_blob_is_none() {
        assert!(from_blob(&[1, 2, 3]).is_none()); // not a multiple of 4
        assert!(from_blob(&[]).is_none());
    }

    #[test]
    fn topn_excludes_self_sorted_and_bounded() {
        let q = vec![1.0, 0.0];
        let entries = vec![
            CacheEntry {
                item_id: 1,
                content_hash: "self".into(),
                vec: vec![1.0, 0.0],
            },
            CacheEntry {
                item_id: 2,
                content_hash: "a".into(),
                vec: vec![0.9, 0.1],
            },
            CacheEntry {
                item_id: 3,
                content_hash: "b".into(),
                vec: vec![0.0, 1.0],
            },
            CacheEntry {
                item_id: 4,
                content_hash: "c".into(),
                vec: vec![0.8, 0.2],
            },
        ];
        let out = top_n(&q, &entries, "self", 2);
        assert_eq!(out.len(), 2);
        // self excluded; strongest first.
        assert_eq!(out[0].item_id, 2);
        assert_eq!(out[1].item_id, 4);
        assert!(out[0].score >= out[1].score);
    }

    #[test]
    fn topn_huge_n_does_not_panic() {
        // `--top` is caller-controlled; a huge value must not overflow the
        // pre-allocation. Returns at most the available entries.
        let q = vec![1.0, 0.0];
        let entries = vec![CacheEntry {
            item_id: 1,
            content_hash: "a".into(),
            vec: vec![1.0, 0.0],
        }];
        let out = top_n(&q, &entries, "", usize::MAX / 2);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn topn_skips_dim_mismatch() {
        let q = vec![1.0, 0.0, 0.0];
        let entries = vec![
            CacheEntry {
                item_id: 1,
                content_hash: "a".into(),
                vec: vec![1.0, 0.0],
            }, // wrong dim
            CacheEntry {
                item_id: 2,
                content_hash: "b".into(),
                vec: vec![1.0, 0.0, 0.0],
            },
        ];
        let out = top_n(&q, &entries, "", 5);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].item_id, 2);
    }
}
