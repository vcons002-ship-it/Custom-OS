//! modeld — the on-device model runtime (docs/06-hybrid-ai.md).
//!
//! M0: announce and heartbeat. M4 links llama.cpp + the embedding models
//! and implements Facet Resolution and the local/escalation router.

fn main() -> anyhow::Result<()> {
    clade_proto::run_service_stub("modeld")
}
