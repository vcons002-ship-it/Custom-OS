//! Query: resolve a path to its vector, rank the cache by cosine, optionally
//! re-rank by capture time, and hydrate results to presentable hits.

use crate::db::{self, ItemBrief};
use crate::vector::{self, CacheEntry};
use anyhow::{anyhow, Result};
use rusqlite::Connection;

/// One search result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryHit {
    pub sub_id: String,
    pub path: String,
    pub score: f32,
    pub capture_ts: Option<i64>,
}

const TIME_COSINE_W: f32 = 0.85;
const TIME_DECAY_W: f32 = 0.15;
const TAU_SECS: f32 = 86_400.0;

/// Top-`top` neighbours of the image at `query_path`. The query image itself
/// is excluded. `time_aware` blends visual similarity with capture-time
/// proximity. Errors if the path is not indexed.
pub fn query_neighbors(
    conn: &Connection,
    cache: &[CacheEntry],
    query_path: &str,
    top: usize,
    time_aware: bool,
) -> Result<Vec<QueryHit>> {
    let (item_id, content_hash) = db::item_id_for_path(conn, query_path)?
        .ok_or_else(|| anyhow!("{query_path} is not indexed"))?;
    let qvec = cache
        .iter()
        .find(|e| e.item_id == item_id)
        .map(|e| e.vec.clone())
        .ok_or_else(|| anyhow!("no embedding for {query_path}"))?;

    // Over-fetch when time-aware so the re-rank has candidates to reorder.
    let fetch = if time_aware { (top * 4).max(top) } else { top };
    let neighbors = vector::top_n(&qvec, cache, &content_hash, fetch);

    let mut hits: Vec<QueryHit> = Vec::new();
    let q_ts = if time_aware {
        db::item_brief(conn, item_id)?.and_then(|b| b.capture_ts)
    } else {
        None
    };
    for nb in neighbors {
        if let Some(b) = db::item_brief(conn, nb.item_id)? {
            let score = if time_aware {
                TIME_COSINE_W * nb.score + TIME_DECAY_W * time_score(q_ts, b.capture_ts)
            } else {
                nb.score
            };
            hits.push(QueryHit {
                sub_id: b.sub_id,
                path: b.path,
                score,
                capture_ts: b.capture_ts,
            });
        }
    }
    if time_aware {
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    hits.truncate(top);
    Ok(hits)
}

/// Exponential decay on capture-time distance; neutral (0.5) when unknown.
fn time_score(a: Option<i64>, b: Option<i64>) -> f32 {
    match (a, b) {
        (Some(x), Some(y)) => (-(x - y).abs() as f32 / TAU_SECS).exp(),
        _ => 0.5,
    }
}

/// Hydrate a list of item briefs (for `list` / `fts`) into hits.
pub fn briefs_to_hits(briefs: Vec<ItemBrief>) -> Vec<QueryHit> {
    briefs
        .into_iter()
        .map(|b| QueryHit {
            sub_id: b.sub_id,
            path: b.path,
            score: 0.0,
            capture_ts: b.capture_ts,
        })
        .collect()
}
