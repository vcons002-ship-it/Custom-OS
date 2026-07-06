//! The M3 exit gate, in code: index ~20 clustered photos, then query the
//! nearest neighbours of one and assert they are same-cluster; plus move,
//! delete, and poison-file robustness.

mod common;

use std::path::Path;
use substrated::db;
use substrated::indexer::Indexer;
use substrated::vector::{self, CacheEntry};
use substrated::PhashHistEmbedder;

fn open(dir: &Path) -> Indexer {
    Indexer::open(
        &dir.join("substrate.db"),
        Box::new(PhashHistEmbedder::new()),
    )
    .unwrap()
}

fn top_neighbors(ix: &Indexer, query_path: &Path, n: usize) -> Vec<String> {
    let conn = ix.connection();
    let (item_id, content_hash) = db::item_id_for_path(conn, &query_path.to_string_lossy())
        .unwrap()
        .unwrap();
    let cache: Vec<CacheEntry> = ix.load_cache().unwrap();
    let qvec = cache
        .iter()
        .find(|e| e.item_id == item_id)
        .unwrap()
        .vec
        .clone();
    let neighbors = vector::top_n(&qvec, &cache, &content_hash, n);
    neighbors
        .iter()
        .map(|nb| db::item_brief(conn, nb.item_id).unwrap().unwrap().path)
        .collect()
}

#[test]
fn indexes_and_finds_same_cluster_neighbors() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("library");
    let groups = common::populate_library(&lib, 5); // 20 images, 4 clusters × 5

    let mut ix = open(tmp.path());
    let report = ix.scan_root(&lib).unwrap();
    assert_eq!(report.indexed, 20, "all 20 unique images indexed");

    let stats = db::stats(ix.connection()).unwrap();
    assert_eq!(
        (stats.items, stats.files, stats.embeddings, stats.failed),
        (20, 20, 20, 0)
    );

    // Query a red-cluster (cluster 0) image; its 4 nearest should be red too.
    let query = &groups[0][0];
    let neighbors = top_neighbors(&ix, query, 4);
    assert_eq!(neighbors.len(), 4);
    for path in &neighbors {
        let base = Path::new(path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        assert!(
            base.starts_with("c0_"),
            "expected a red-cluster neighbor, got {base}"
        );
    }
}

#[test]
fn move_keeps_one_item_and_updates_path() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("library");
    common::populate_library(&lib, 3); // 12 images
    let mut ix = open(tmp.path());
    ix.scan_root(&lib).unwrap();
    let before = db::stats(ix.connection()).unwrap().items;

    // Rename one file (same bytes) and rescan.
    let old = lib.join("c0_v0.png");
    let new = lib.join("moved.png");
    std::fs::rename(&old, &new).unwrap();
    ix.scan_root(&lib).unwrap();

    let after = db::stats(ix.connection()).unwrap();
    assert_eq!(
        after.items, before,
        "a move must not create or drop an item"
    );
    assert!(
        db::item_id_for_path(ix.connection(), &old.to_string_lossy())
            .unwrap()
            .is_none()
    );
    assert!(
        db::item_id_for_path(ix.connection(), &new.to_string_lossy())
            .unwrap()
            .is_some()
    );
}

#[test]
fn delete_removes_the_item() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("library");
    common::populate_library(&lib, 3);
    let mut ix = open(tmp.path());
    ix.scan_root(&lib).unwrap();
    let before = db::stats(ix.connection()).unwrap().items;

    std::fs::remove_file(lib.join("c1_v0.png")).unwrap();
    let report = ix.scan_root(&lib).unwrap();
    assert_eq!(report.removed, 1);
    assert_eq!(db::stats(ix.connection()).unwrap().items, before - 1);
}

#[test]
fn corrupt_file_is_quarantined_not_fatal() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("library");
    common::populate_library(&lib, 2); // 8 valid
    std::fs::write(lib.join("broken.jpg"), b"\xff\xd8\xff not a real jpeg").unwrap();

    let mut ix = open(tmp.path());
    let report = ix.scan_root(&lib).unwrap();
    assert_eq!(report.indexed, 8, "valid images still indexed");
    assert_eq!(
        report.failed, 1,
        "the corrupt file is a failure, not a crash"
    );

    let stats = db::stats(ix.connection()).unwrap();
    assert_eq!(stats.items, 8);
    assert_eq!(stats.failed, 1);

    // A rescan does not retry the quarantined file (unchanged stat).
    let report2 = ix.scan_root(&lib).unwrap();
    assert_eq!(report2.failed, 0, "quarantined file skipped on rescan");
    assert_eq!(report2.unchanged, 8);
}
