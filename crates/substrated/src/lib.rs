//! substrated — the Substrate: Clade's semantic filesystem indexer
//! (docs/04-ai-architecture.md §6, docs/08-data-knowledge-model.md).
//!
//! Logic lives in library modules so integration tests can drive indexing and
//! search directly, without the daemon's threads.

pub mod bus;
pub mod cli;
pub mod config;
pub mod ctl;
pub mod daemon;
pub mod db;
pub mod embed;
pub mod exif;
pub mod hashid;
pub mod indexer;
pub mod meta_text;
pub mod search;
pub mod vector;
pub mod watch;

pub use embed::{Embedder, Embedding, PhashHistEmbedder};
pub use indexer::{Indexer, Outcome, ScanReport};

/// One console log line (stdout is the console when running under the image).
pub fn log(who: &str, msg: &str) {
    println!("[clade:{who}] {msg}");
}
