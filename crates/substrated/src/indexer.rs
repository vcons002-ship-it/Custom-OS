//! The single-writer indexer: owns the read-write Connection (the sole writer)
//! and turns a filesystem path into rows across items/files/embeddings/item_fts
//! in one transaction. Change detection uses the `files` stat-cache; identity
//! is the blake3 content hash, so byte-identical copies dedup to one item and
//! a move/rename resolves to the same item.

use crate::db;
use crate::embed::Embedder;
use crate::vector::{self, CacheEntry};
use crate::{config, exif, hashid, meta_text};
use anyhow::Result;
use rusqlite::Connection;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use walkdir::WalkDir;

/// What happened to one path during indexing.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// A brand-new item (new content hash) was created.
    Indexed,
    /// The path existed but now points at different content.
    Updated,
    /// Byte-identical content already indexed (dedup or move); item reused.
    Deduped,
    /// (size, mtime) unchanged — nothing done but the seen_ts bump.
    Unchanged,
    /// Decode/embed failed; quarantined in `failed_files`.
    Failed(String),
    /// Not a candidate (vanished, too large, or wrong type).
    Skipped(String),
}

/// Aggregate scan result.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScanReport {
    pub indexed: usize,
    pub updated: usize,
    pub deduped: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub removed: usize,
}

/// Owns the writer connection + the embedder.
pub struct Indexer {
    conn: Connection,
    embedder: Box<dyn Embedder>,
    /// Strictly-monotonic ns stamp for `files.seen_ts`, so the delete-sweep's
    /// `seen_ts < scan_ts` is correct even for scans in the same wall second.
    last_stamp: i64,
}

impl Indexer {
    pub fn open(db_path: &Path, embedder: Box<dyn Embedder>) -> Result<Self> {
        let conn = db::open_rw(db_path)?;
        // Keep meta.embed_model in sync with the active embedder.
        db::set_meta(&conn, "embed_model", embedder.model_id())?;
        // Resume monotonicity across restarts.
        let last_stamp: i64 =
            conn.query_row("SELECT COALESCE(MAX(seen_ts),0) FROM files", [], |r| {
                r.get(0)
            })?;
        Ok(Self {
            conn,
            embedder,
            last_stamp,
        })
    }

    /// A strictly-increasing scan/upsert stamp (ns, never repeats).
    pub fn next_stamp(&mut self) -> i64 {
        let s = db::now_ns().max(self.last_stamp + 1);
        self.last_stamp = s;
        s
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// The in-memory search cache for the active model.
    pub fn load_cache(&self) -> Result<Vec<CacheEntry>> {
        db::load_cache(&self.conn, self.embedder.model_id())
    }

    /// The active embedding model id.
    pub fn model_id(&self) -> &'static str {
        self.embedder.model_id()
    }

    /// Truncate the WAL back into the main db (called on clean shutdown).
    pub fn checkpoint(&self) {
        let _ = self.conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
    }

    /// Index one path. `seen_ts` marks it live for the scan delete-sweep.
    pub fn index_path(&mut self, path: &Path, seen_ts: i64) -> Outcome {
        match self.try_index_path(path, seen_ts) {
            Ok(o) => o,
            Err(e) => Outcome::Skipped(e.to_string()),
        }
    }

    fn try_index_path(&mut self, path: &Path, seen_ts: i64) -> Result<Outcome> {
        let path_str = path.to_string_lossy().to_string();

        let meta = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return Ok(Outcome::Skipped("vanished".into())),
        };
        if !meta.is_file() {
            return Ok(Outcome::Skipped("not a file".into()));
        }
        let size = meta.len() as i64;
        let mtime_ns = meta.mtime() * 1_000_000_000 + meta.mtime_nsec();
        let (dev, inode) = (meta.dev() as i64, meta.ino() as i64);

        // Stat-cache fast path: unchanged file → just bump liveness.
        if let Some((f_size, f_mtime)) = self.file_stat(&path_str)? {
            if f_size == size && f_mtime == mtime_ns {
                self.conn.execute(
                    "UPDATE files SET seen_ts=?1 WHERE path=?2",
                    (seen_ts, &path_str),
                )?;
                return Ok(Outcome::Unchanged);
            }
        }

        // Quarantine cache: a known-bad file with unchanged stat is skipped;
        // a changed stat (e.g. a completed partial copy) invalidates it.
        if let Some((q_size, q_mtime)) = self.failed_stat(&path_str)? {
            if q_size == size && q_mtime == mtime_ns {
                return Ok(Outcome::Skipped("quarantined".into()));
            }
            self.conn
                .execute("DELETE FROM failed_files WHERE path=?1", [&path_str])?;
        }

        if size as u64 > config::MAX_BYTES {
            self.quarantine(&path_str, size, mtime_ns, "exceeds byte cap")?;
            return Ok(Outcome::Failed("too large".into()));
        }
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(_) => return Ok(Outcome::Skipped("read failed".into())),
        };
        let content_hash = hashid::hash_bytes(&bytes);

        // What (if anything) did this path map to before?
        let prior = db::item_id_for_path(&self.conn, &path_str)?;

        let tx = self.conn.transaction()?;
        let outcome;
        let item_id = if let Some(id) = item_id_by_hash(&tx, &content_hash)? {
            // Content already known — dedup or move. No embed needed.
            outcome = match &prior {
                Some((pid, phash)) if *phash == content_hash => Outcome::Unchanged,
                Some(_) => Outcome::Updated,
                None => Outcome::Deduped,
            };
            id
        } else {
            // New content: embed (poison-marker guards a panic under abort).
            db::set_meta(&tx, "indexing_path", &path_str)?;
            let embedding = match self.embedder.embed(&bytes) {
                Ok(e) => e,
                Err(e) => {
                    db::set_meta(&tx, "indexing_path", "")?;
                    tx.commit()?;
                    self.quarantine(&path_str, size, mtime_ns, &e.to_string())?;
                    return Ok(Outcome::Failed(e.to_string()));
                }
            };
            db::set_meta(&tx, "indexing_path", "")?;
            let ex = exif::read(&bytes);
            let now = db::now_secs();
            let sub_id = hashid::sub_id_for(&content_hash);
            tx.execute(
                "INSERT INTO items(sub_id,content_hash,facet,format,width,height,byte_len,
                                   capture_ts,first_seen_ts,indexed_ts)
                 VALUES(?1,?2,'image',?3,?4,?5,?6,?7,?8,?8)",
                rusqlite::params![
                    sub_id,
                    content_hash,
                    embedding.format,
                    embedding.width as i64,
                    embedding.height as i64,
                    size,
                    ex.capture_ts,
                    now,
                ],
            )?;
            let id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO embeddings(item_id,model,dim,vec,created_ts) VALUES(?1,?2,?3,?4,?5)",
                rusqlite::params![
                    id,
                    self.embedder.model_id(),
                    embedding.vec.len() as i64,
                    vector::to_blob(&embedding.vec),
                    now,
                ],
            )?;
            tx.execute(
                "INSERT INTO item_fts(rowid,filename,text) VALUES(?1,?2,?3)",
                rusqlite::params![
                    id,
                    meta_text::filename_field(path),
                    meta_text::text_field(path, ex.make.as_deref(), ex.model.as_deref()),
                ],
            )?;
            // A new item, but if this path previously held other content it is
            // an update of that location, not a fresh discovery.
            outcome = if prior.is_some() {
                Outcome::Updated
            } else {
                Outcome::Indexed
            };
            id
        };

        // Point this path at the resolved item.
        tx.execute(
            "INSERT INTO files(path,item_id,size,mtime_ns,dev,inode,seen_ts)
             VALUES(?1,?2,?3,?4,?5,?6,?7)
             ON CONFLICT(path) DO UPDATE SET
               item_id=excluded.item_id, size=excluded.size, mtime_ns=excluded.mtime_ns,
               dev=excluded.dev, inode=excluded.inode, seen_ts=excluded.seen_ts",
            rusqlite::params![path_str, item_id, size, mtime_ns, dev, inode, seen_ts],
        )?;

        // If the path was displaced from an older item, GC it if now orphaned.
        if let Some((old_id, old_hash)) = prior {
            if old_hash != content_hash {
                gc_item_if_orphan(&tx, old_id)?;
            }
        }
        tx.commit()?;
        Ok(outcome)
    }

    /// Remove a path; GC the item if it was its last location. Returns true if
    /// the path was present.
    pub fn remove_path(&mut self, path: &Path) -> Result<bool> {
        let path_str = path.to_string_lossy().to_string();
        let prior = db::item_id_for_path(&self.conn, &path_str)?;
        let tx = self.conn.transaction()?;
        let removed = tx.execute("DELETE FROM files WHERE path=?1", [&path_str])? > 0;
        if let Some((item_id, _)) = prior {
            gc_item_if_orphan(&tx, item_id)?;
        }
        tx.commit()?;
        Ok(removed)
    }

    /// Full reconcile scan: index every candidate under `root`, then delete
    /// rows for paths that disappeared, and prune stale quarantine entries.
    pub fn scan_root(&mut self, root: &Path) -> Result<ScanReport> {
        let scan_ts = self.next_stamp();
        let mut report = ScanReport::default();
        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() || !config::has_image_ext(entry.path()) {
                continue;
            }
            match self.index_path(entry.path(), scan_ts) {
                Outcome::Indexed => report.indexed += 1,
                Outcome::Updated => report.updated += 1,
                Outcome::Deduped => report.deduped += 1,
                Outcome::Unchanged => report.unchanged += 1,
                Outcome::Failed(_) => report.failed += 1,
                Outcome::Skipped(_) => {}
            }
        }
        report.removed = self.sweep_deleted(root, scan_ts)?;
        Ok(report)
    }

    /// Delete `files` rows under `root` not touched this scan, GC orphaned
    /// items, and drop quarantine entries whose file is gone.
    fn sweep_deleted(&mut self, root: &Path, scan_ts: i64) -> Result<usize> {
        let root_str = root.to_string_lossy().to_string();
        let like = format!("{}%", escape_like(&format!("{root_str}/")));
        let tx = self.conn.transaction()?;

        let mut stale: Vec<i64> = Vec::new();
        {
            let mut stmt = tx.prepare(
                "SELECT DISTINCT item_id FROM files
                 WHERE seen_ts < ?1 AND (path = ?2 OR path LIKE ?3 ESCAPE '\\')",
            )?;
            let mut rows = stmt.query(rusqlite::params![scan_ts, root_str, like])?;
            while let Some(row) = rows.next()? {
                stale.push(row.get(0)?);
            }
        }
        let removed = tx.execute(
            "DELETE FROM files WHERE seen_ts < ?1 AND (path = ?2 OR path LIKE ?3 ESCAPE '\\')",
            rusqlite::params![scan_ts, root_str, like],
        )?;
        for id in stale {
            gc_item_if_orphan(&tx, id)?;
        }
        tx.commit()?;
        Ok(removed)
    }

    fn file_stat(&self, path: &str) -> Result<Option<(i64, i64)>> {
        Ok(self
            .conn
            .query_row(
                "SELECT size,mtime_ns FROM files WHERE path=?1",
                [path],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
            )
            .ok())
    }

    fn failed_stat(&self, path: &str) -> Result<Option<(i64, i64)>> {
        Ok(self
            .conn
            .query_row(
                "SELECT size,mtime_ns FROM failed_files WHERE path=?1",
                [path],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
            )
            .ok())
    }

    fn quarantine(&self, path: &str, size: i64, mtime_ns: i64, err: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO failed_files(path,size,mtime_ns,error,failed_ts) VALUES(?1,?2,?3,?4,?5)
             ON CONFLICT(path) DO UPDATE SET size=excluded.size, mtime_ns=excluded.mtime_ns,
               error=excluded.error, failed_ts=excluded.failed_ts",
            rusqlite::params![path, size, mtime_ns, err, db::now_secs()],
        )?;
        Ok(())
    }
}

fn item_id_by_hash(conn: &Connection, content_hash: &str) -> Result<Option<i64>> {
    Ok(conn
        .query_row(
            "SELECT id FROM items WHERE content_hash=?1",
            [content_hash],
            |r| r.get(0),
        )
        .ok())
}

/// Delete an item if it has no remaining file locations. item_fts is a virtual
/// table (no FK cascade), so its row is removed explicitly; files/embeddings
/// cascade via the schema's foreign keys.
fn gc_item_if_orphan(conn: &Connection, item_id: i64) -> Result<()> {
    let orphan: bool = conn
        .query_row(
            "SELECT NOT EXISTS(SELECT 1 FROM files WHERE item_id=?1)",
            [item_id],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if orphan {
        conn.execute("DELETE FROM item_fts WHERE rowid=?1", [item_id])?;
        conn.execute("DELETE FROM items WHERE id=?1", [item_id])?;
    }
    Ok(())
}

/// Escape LIKE metacharacters (%, _, \) so a path is matched literally.
fn escape_like(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '%' | '_' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}
