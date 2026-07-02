//! cortexd — the Cortex (docs/04-ai-architecture.md §1).
//!
//! M0: announce and heartbeat. The Intent/Plan loop lands at M6, fed by
//! focus events from the Weave (M5) and facets from modeld (M4).

fn main() -> anyhow::Result<()> {
    clade_proto::run_service_stub("cortexd")
}
