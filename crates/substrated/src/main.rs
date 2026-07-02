//! substrated — the Substrate indexer (docs/04-ai-architecture.md §6).
//!
//! M0: announce and heartbeat. M3 adds the folder watcher and the
//! SQLite (metadata + FTS + sqlite-vec) index.

fn main() -> anyhow::Result<()> {
    clade_proto::run_service_stub("substrated")
}
