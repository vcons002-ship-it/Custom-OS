//! SQLite: schema, connections, migration, and read queries.
//!
//! Identity is content (`items.content_hash`); `files` maps many paths to one
//! item (dedup) and doubles as the stat-cache for change detection. Vectors
//! are model-tagged so M3's `phash-hist-v1` and M4's CLIP coexist. FTS5 is a
//! regular table maintained by the single writer in the same transaction.

use crate::vector::CacheEntry;
use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const SCHEMA_VERSION: i64 = 1;
pub const EMBED_MODEL: &str = "phash-hist-v1";
pub const EMBED_DIM: usize = 160;

const DDL: &str = r#"
CREATE TABLE IF NOT EXISTS meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS items (
  id            INTEGER PRIMARY KEY,
  sub_id        TEXT NOT NULL UNIQUE,
  content_hash  TEXT NOT NULL UNIQUE,
  facet         TEXT NOT NULL DEFAULT 'image',
  format        TEXT,
  width         INTEGER,
  height        INTEGER,
  byte_len      INTEGER NOT NULL,
  capture_ts    INTEGER,
  first_seen_ts INTEGER NOT NULL,
  indexed_ts    INTEGER NOT NULL,
  facets_json     TEXT NOT NULL DEFAULT '[{"kind":"image","confidence":1.0}]',
  provenance_json TEXT NOT NULL DEFAULT '{}',
  entities_json   TEXT NOT NULL DEFAULT '[]'
) STRICT;
CREATE INDEX IF NOT EXISTS items_capture ON items(capture_ts);

CREATE TABLE IF NOT EXISTS files (
  path     TEXT PRIMARY KEY,
  item_id  INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  size     INTEGER NOT NULL,
  mtime_ns INTEGER NOT NULL,
  dev      INTEGER NOT NULL DEFAULT 0,
  inode    INTEGER NOT NULL DEFAULT 0,
  seen_ts  INTEGER NOT NULL
) STRICT;
CREATE INDEX IF NOT EXISTS files_by_item  ON files(item_id);
CREATE INDEX IF NOT EXISTS files_by_inode ON files(dev, inode);

CREATE TABLE IF NOT EXISTS embeddings (
  item_id    INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  model      TEXT NOT NULL,
  dim        INTEGER NOT NULL,
  vec        BLOB NOT NULL,
  created_ts INTEGER NOT NULL,
  PRIMARY KEY (item_id, model)
) STRICT;
CREATE INDEX IF NOT EXISTS embeddings_by_model ON embeddings(model);

CREATE TABLE IF NOT EXISTS failed_files (
  path      TEXT PRIMARY KEY,
  size      INTEGER NOT NULL,
  mtime_ns  INTEGER NOT NULL,
  error     TEXT NOT NULL,
  failed_ts INTEGER NOT NULL
) STRICT;

CREATE VIRTUAL TABLE IF NOT EXISTS item_fts USING fts5(
  filename,
  text,
  tokenize = 'unicode61 remove_diacritics 2'
);
"#;

/// Seconds since the Unix epoch (wall clock).
pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Nanoseconds since the Unix epoch — the resolution used for `files.seen_ts`
/// scan generations (seconds collide across rapid rescans).
pub fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0)
}

/// Open the read-write (indexer) connection, run PRAGMAs + migration, and
/// prove the bundled amalgamation actually has FTS5 + STRICT.
pub fn open_rw(path: &Path) -> Result<Connection> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }
    let conn = Connection::open(path)
        .with_context(|| format!("opening substrate db at {}", path.display()))?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         PRAGMA foreign_keys=ON;
         PRAGMA busy_timeout=5000;
         PRAGMA wal_autocheckpoint=1000;
         PRAGMA temp_store=MEMORY;",
    )?;
    self_test(&conn)?;
    migrate(&conn)?;
    Ok(conn)
}

/// Open a read-only connection for the CLI / control-socket handlers.
pub fn open_ro(path: &Path) -> Result<Connection> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .with_context(|| format!("opening substrate db (ro) at {}", path.display()))?;
    conn.busy_timeout(std::time::Duration::from_millis(5000))?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

/// Fail loudly if the bundled SQLite lacks FTS5 or STRICT, rather than
/// silently degrading at runtime inside the image.
fn self_test(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS temp._clade_fts_probe USING fts5(x);
         CREATE TABLE IF NOT EXISTS temp._clade_strict_probe (x INTEGER) STRICT;",
    )
    .context("bundled SQLite lacks FTS5 or STRICT support")?;
    Ok(())
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(DDL)?;
    // Seed identity rows once.
    set_meta(conn, "schema_version", &SCHEMA_VERSION.to_string())?;
    set_meta_if_absent(conn, "embed_model", EMBED_MODEL)?;
    set_meta_if_absent(conn, "embed_dim", &EMBED_DIM.to_string())?;
    Ok(())
}

pub fn set_meta(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO meta(key,value) VALUES(?1,?2)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        (key, value),
    )?;
    Ok(())
}

fn set_meta_if_absent(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO meta(key,value) VALUES(?1,?2)",
        (key, value),
    )?;
    Ok(())
}

pub fn get_meta(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM meta WHERE key=?1", [key], |r| {
            r.get::<_, String>(0)
        })
        .ok())
}

/// Load every embedding for `model` into the in-memory search cache.
pub fn load_cache(conn: &Connection, model: &str) -> Result<Vec<CacheEntry>> {
    let mut stmt = conn.prepare(
        "SELECT e.item_id, i.content_hash, e.vec
         FROM embeddings e JOIN items i ON i.id = e.item_id
         WHERE e.model = ?1",
    )?;
    let rows = stmt.query_map([model], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Vec<u8>>(2)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (item_id, content_hash, blob) = row?;
        if let Some(vec) = crate::vector::from_blob(&blob) {
            out.push(CacheEntry {
                item_id,
                content_hash,
                vec,
            });
        }
    }
    Ok(out)
}

/// A row for `list` / query result resolution.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ItemBrief {
    pub sub_id: String,
    pub content_hash: String,
    pub path: String,
    pub capture_ts: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

/// Resolve an item id to its brief (one representative path).
pub fn item_brief(conn: &Connection, item_id: i64) -> Result<Option<ItemBrief>> {
    let brief = conn
        .query_row(
            "SELECT i.sub_id, i.content_hash,
                    COALESCE((SELECT path FROM files WHERE item_id=i.id ORDER BY path LIMIT 1), ''),
                    i.capture_ts, i.width, i.height
             FROM items i WHERE i.id=?1",
            [item_id],
            |r| {
                Ok(ItemBrief {
                    sub_id: r.get(0)?,
                    content_hash: r.get(1)?,
                    path: r.get(2)?,
                    capture_ts: r.get(3)?,
                    width: r.get(4)?,
                    height: r.get(5)?,
                })
            },
        )
        .ok();
    Ok(brief)
}

/// Look up an item id + content hash by a path currently in the index.
pub fn item_id_for_path(conn: &Connection, path: &str) -> Result<Option<(i64, String)>> {
    let row = conn
        .query_row(
            "SELECT i.id, i.content_hash FROM files f JOIN items i ON i.id=f.item_id WHERE f.path=?1",
            [path],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
        )
        .ok();
    Ok(row)
}

/// The most recent `limit` items (newest first).
pub fn list_items(conn: &Connection, limit: usize) -> Result<Vec<ItemBrief>> {
    let mut stmt =
        conn.prepare("SELECT i.id FROM items i ORDER BY i.indexed_ts DESC, i.id DESC LIMIT ?1")?;
    let ids: Vec<i64> = stmt
        .query_map([limit as i64], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = Vec::new();
    for id in ids {
        if let Some(b) = item_brief(conn, id)? {
            out.push(b);
        }
    }
    Ok(out)
}

/// FTS search returning item briefs.
pub fn fts_search(conn: &Connection, query: &str, limit: usize) -> Result<Vec<ItemBrief>> {
    let mut stmt =
        conn.prepare("SELECT rowid FROM item_fts WHERE item_fts MATCH ?1 ORDER BY rank LIMIT ?2")?;
    let ids: Vec<i64> = stmt
        .query_map(rusqlite::params![query, limit as i64], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = Vec::new();
    for id in ids {
        if let Some(b) = item_brief(conn, id)? {
            out.push(b);
        }
    }
    Ok(out)
}

/// Aggregate counts for `stats`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Stats {
    pub items: i64,
    pub files: i64,
    pub embeddings: i64,
    pub failed: i64,
    pub schema_version: String,
    pub embed_model: String,
}

pub fn stats(conn: &Connection) -> Result<Stats> {
    let count = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
    Ok(Stats {
        items: count("SELECT count(*) FROM items")?,
        files: count("SELECT count(*) FROM files")?,
        embeddings: count("SELECT count(*) FROM embeddings")?,
        failed: count("SELECT count(*) FROM failed_files")?,
        schema_version: get_meta(conn, "schema_version")?.unwrap_or_default(),
        embed_model: get_meta(conn, "embed_model")?.unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_opens_idempotent_and_strict() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.db");
        let c = open_rw(&path).unwrap();
        // Reopen is a no-op.
        drop(c);
        let c = open_rw(&path).unwrap();

        // WAL is set.
        let mode: String = c
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(mode.to_lowercase(), "wal");

        // Identity rows present.
        assert_eq!(
            get_meta(&c, "schema_version").unwrap().as_deref(),
            Some("1")
        );
        assert_eq!(
            get_meta(&c, "embed_model").unwrap().as_deref(),
            Some(EMBED_MODEL)
        );

        // STRICT rejects a wrong-typed insert.
        let bad = c.execute(
            "INSERT INTO items(sub_id,content_hash,byte_len,first_seen_ts,indexed_ts)
             VALUES('s','h','not-an-int',0,0)",
            [],
        );
        assert!(
            bad.is_err(),
            "STRICT should reject a TEXT into an INTEGER column"
        );
    }
}
