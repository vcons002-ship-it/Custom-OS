//! gated — the only path out of the machine (docs/09-privacy-trust.md).
//!
//! M0: announce and heartbeat. M6 adds the provider-agnostic escalation
//! client (owner's Ollama server by default, Gemini API as opt-in fallback)
//! behind the Redaction Gate stub, with full disclosure on the bus.

fn main() -> anyhow::Result<()> {
    clade_proto::run_service_stub("gated")
}
