//! Indexer correctness: dedup, change detection, modify, FTS, model filtering.

mod common;

use std::path::Path;
use substrated::db;
use substrated::indexer::{Indexer, Outcome};
use substrated::PhashHistEmbedder;

fn open(dir: &Path) -> Indexer {
    Indexer::open(
        &dir.join("substrate.db"),
        Box::new(PhashHistEmbedder::new()),
    )
    .unwrap()
}

#[test]
fn dedup_same_bytes_two_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("lib");
    let img = common::cluster_image(0, 0);
    common::write_png(&img, &lib.join("a.png"));
    common::write_png(&img, &lib.join("b.png")); // byte-identical copy

    let mut ix = open(tmp.path());
    ix.scan_root(&lib).unwrap();

    let s = db::stats(ix.connection()).unwrap();
    assert_eq!(s.items, 1, "identical bytes → one item");
    assert_eq!(s.files, 2, "two locations");
    assert_eq!(s.embeddings, 1, "one embedding");
}

#[test]
fn change_detection_skips_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("lib");
    common::write_png(&common::cluster_image(1, 0), &lib.join("x.png"));
    let mut ix = open(tmp.path());

    let r1 = ix.scan_root(&lib).unwrap();
    assert_eq!(r1.indexed, 1);
    let r2 = ix.scan_root(&lib).unwrap();
    assert_eq!(r2.indexed, 0);
    assert_eq!(r2.unchanged, 1, "second scan re-embeds nothing");
}

#[test]
fn modify_updates_content_hash_and_gcs_orphan() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("lib");
    let p = lib.join("photo.png");
    common::write_png(&common::cluster_image(0, 0), &p);
    let mut ix = open(tmp.path());
    ix.scan_root(&lib).unwrap();
    let hash1 = db::item_id_for_path(ix.connection(), &p.to_string_lossy())
        .unwrap()
        .unwrap()
        .1;

    // Overwrite with different content; bump mtime so change detection fires.
    std::thread::sleep(std::time::Duration::from_millis(10));
    common::write_png(&common::cluster_image(2, 5), &p);
    let ts = ix.next_stamp();
    let out = ix.index_path(&p, ts);
    assert_eq!(out, Outcome::Updated);

    let hash2 = db::item_id_for_path(ix.connection(), &p.to_string_lossy())
        .unwrap()
        .unwrap()
        .1;
    assert_ne!(hash1, hash2, "content hash changed");
    // The old item was orphaned and GC'd → still exactly one item.
    assert_eq!(db::stats(ix.connection()).unwrap().items, 1);
}

#[test]
fn fts_matches_filename() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("lib");
    common::write_png(&common::cluster_image(0, 0), &lib.join("beach_sunset.png"));
    common::write_png(&common::cluster_image(1, 0), &lib.join("office_desk.png"));
    let mut ix = open(tmp.path());
    ix.scan_root(&lib).unwrap();

    let hits = db::fts_search(ix.connection(), "sunset", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].path.ends_with("beach_sunset.png"));

    let none = db::fts_search(ix.connection(), "mountain", 10).unwrap();
    assert!(none.is_empty());
}

#[test]
fn search_filters_by_model() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("lib");
    common::write_png(&common::cluster_image(0, 0), &lib.join("a.png"));
    let mut ix = open(tmp.path());
    ix.scan_root(&lib).unwrap();

    // A foreign-model embedding must be invisible to the phash cache.
    ix.connection()
        .execute(
            "INSERT INTO embeddings(item_id,model,dim,vec,created_ts)
             SELECT id,'clip-vit-b32',3,X'000000000000000000000000',0 FROM items LIMIT 1",
            [],
        )
        .unwrap();
    let cache = ix.load_cache().unwrap();
    assert_eq!(cache.len(), 1, "phash cache ignores the clip-model row");
}
