//! capd — the Capability registry and materializer (docs/06-hybrid-ai.md).
//!
//! M0: announce and heartbeat. M5 loads the manifests in capabilities/,
//! validates them, and hands the Weave real Surfaces from sandboxed workers.

fn main() -> anyhow::Result<()> {
    clade_proto::run_service_stub("capd")
}
