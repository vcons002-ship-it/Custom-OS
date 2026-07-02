# 10 — Build-Phase Tech Stack

Grounded choices for the build phase, with rationale and the alternative each was chosen
over. Guiding principle: **local-first, embeddable, swappable models, MCP-shaped tools.**
Nothing here is built yet; this is the bill of materials the roadmap draws from.

| Layer | Choice | Rationale (and the alternative) |
|---|---|---|
| **OS image** | **Buildroot** minimal Linux image; signed, immutable, A/B slots | Buildroot over Yocto: radically simpler for a single fixed image with a tiny service set; Yocto's layer machinery earns its complexity only with hardware-variant proliferation (revisit at Phase 3). |
| **Kernel** | Mainline Linux, trimmed config, `quiet` boot | Hardware layer only ([03-os-layer.md](03-os-layer.md)). Driver coverage is the entire reason not to write a kernel. |
| **Init / supervisor** | Custom Rust `weaved` (PID 1) | systemd replaced: Morph's service graph is six fixed services; a purpose-built ~runit-scope init keeps PID 1 auditable and boots straight into the Weave. |
| **Compositor / display** | Direct DRM/KMS + GBM rendering; **wlroots as a library** for output/input plumbing | There are no client apps, so a client-serving compositor is the wrong shape. Smithay (pure Rust) is the alternative if wlroots FFI grates. |
| **Weave UI runtime** | **wgpu** (Rust, GPU-native) with a retained scene graph tuned for Materialize/Dissolve | The morphing canvas is animation-first; wgpu gives full control and no web-engine weight at PID-1 distance. Fallback: an embedded web engine (the mockups' HTML/CSS carries straight over) if UI iteration speed dominates — decide at Phase 1 exit. |
| **Services** | Rust throughout (`cortexd`, `substrated`, `gated`, `capd`, `dreamd`) | Memory safety at OS altitude; one language across the mind plane; first-class sandboxing hooks. |
| **On-device LLM** | **llama.cpp** embedded (as a library in `modeld`), 3–8B-class instruction-tuned model, GGUF, baked into the image | llama.cpp over Ollama: Ollama is a developer convenience server; an OS wants the inference engine linked in, versioned with the image. Model is swappable by design — evaluate at build time. |
| **Embeddings** | Compact local embedding model (text + CLIP-class image) in `modeld` | Powers Substrate vectors and Foresight similarity fully offline. |
| **Cloud reasoning** | **Claude API** | Frontier multi-step reasoning + native MCP tool use maps directly onto Capability manifests. Model IDs, pricing, and context limits are deliberately *not* pinned here — verify against the live `claude-api` reference at build time. |
| **Tool protocol** | **MCP** (Model Context Protocol) | Capability `tools` are MCP-shaped ([06-hybrid-ai.md](06-hybrid-ai.md)); one spec serves local invocation, cloud invocation, and a third-party ecosystem. |
| **Stores** | **SQLite** everywhere: metadata, Context Graph (edge table), FTS5 full-text, **sqlite-vec** for vectors; Journal as append-only SQLite WAL-mode DB | Embedded, zero-server, one backup story, battle-tested at OS altitude. Graph DB and vector-DB servers are swaps-if-proven-necessary, not defaults. |
| **Sandboxing** | Namespaces + seccomp + cgroups via the service supervisor; Capability workers get declared mounts/scopes only | Matches the manifest permission model without a container runtime's weight. |
| **First target** | **QEMU/KVM + virtio-gpu** | Reference machine for Phase 1; real-hardware bring-up (one well-supported x86 laptop) is a Phase-2 gate. |

## Two decisions deliberately deferred

1. **wgpu vs. embedded web engine for the Weave** — decided at the end of Phase 1 by
   prototyping the Materialize/Dissolve animation set in both. Criteria: animation
   fidelity, iteration speed, memory footprint, input latency.
2. **Exact model selections** (on-device LLM, embedding models, Claude model IDs) —
   decided at build time against current benchmarks and the live API reference; the
   architecture requires only that they are swappable, which the `modeld` boundary and
   the router guarantee.

---
*Next: [11-roadmap.md](11-roadmap.md) — the phases and their gates.*
