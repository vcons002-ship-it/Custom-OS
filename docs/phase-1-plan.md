# Phase 1 — The Bootable Seed (engineering plan)

The smallest artifact that is honestly this OS: it boots as its own operating system,
has a resident (if modest) mind, and morphs around one content mode. Everything here
serves the single **exit gate** below; anything not on that path is Phase 2.

## Exit gate (the demo that ends Phase 1)

> Cold-boot the image in QEMU to the idle Weave. Open a real photo from a watched folder.
> The correct tool Surfaces materialize around it. One honest related-content suggestion
> appears in the Foresight Rail with an expandable *why*. Make an edit and undo it from
> the Journal. Trigger one escalated reasoning call (image description) and watch it
> round-trip through the Redaction Gate with its reason shown.

Six capabilities, end to end: **boot · morph · index · predict · undo · one gated escalation.**

## Scope

- **One content mode: Images** *(confirmed by the owner)*. Chosen to de-risk: a wrong
  related-photo is shrugged off; a wrong code edit is not. Visually undeniable in a demo.
- **One machine: QEMU/KVM** with virtio-gpu, on the owner's box (RTX 5090 · modern AMD
  CPU · 32GB RAM). Reference VM: **8 vCPUs, 12–16GB RAM, KVM-accelerated** — a true boot
  of the real image (UEFI → kernel → `weaved` → Weave), just on virtual hardware. No
  real-hardware bring-up (that is a Phase 2 gate); GPU passthrough of the 5090 into the
  VM is a documented later option, not Phase 1.
- **Single user, no accounts.** The machine is the person's computer.
- Foresight does **related-content only** (next-action habit prediction is Phase 2).
- The resident-mind depth of Phase 2 (the Current across sessions, Dreamtime, the Workbench,
  Curiosity, Delegation) is **out of scope** — but the Journal and per-focus Intent ship now,
  because the exit gate needs them and they are the spine everything later hangs on.

## Concrete decisions to lock

| Decision | Phase-1 call | Notes |
|---|---|---|
| OS build system | **Buildroot** external tree | Yocto deferred to Phase 3 (hardware variants). |
| Language | **Rust** across all services + `weaved` | One toolchain, sandbox-friendly. |
| Weave renderer | **DECIDED at M2: the custom software compositor** (`crates/weave/src/fb.rs`) — damage-driven CPU rendering into the DRM dumb buffer, fontdue text, raw evdev input | Evidence beat the original wgpu-vs-web-engine framing: the VM has no GPU acceleration (wgpu would drag in a software-Mesa stack), an embedded web engine would reintroduce a userland, and M1/M2 proved CPU compositing handles the Weave's calm UI in a ~1.6MB static binary at 30fps. Revisit GPU (virgl or passthrough) when animation complexity demands it. |
| Inter-service IPC | **Cap'n Proto RPC over Unix domain sockets** | Schema-first messages shared via the `clade-proto` crate; fast, typed, no broker. |
| Event bus | A small pub/sub in `weaved` over the same sockets | Focus events, materialize/dissolve, journal appends. |
| On-device LLM | A 3–4B-class instruction model (GGUF) via **llama.cpp** linked into `modeld` | Exact model chosen at M4 against current benchmarks; swappable by design. |
| Embeddings | A compact CLIP-class image model + small text embedder in `modeld` | Powers Substrate vectors + Foresight similarity. |
| Stores | **SQLite**: metadata + Context Graph (edge table) + FTS5, **sqlite-vec** for vectors; Journal as append-only WAL DB | Embedded, one backup story. |
| Escalation tier | **Owner's Ollama server** (host GPU: RTX 5090 running a Gemma-27B-class model) as the default heavy engine, reached through `gated` over the VM's virtual network; **Gemini API** as the opt-in true-cloud fallback | No Claude API available in this deployment; the router/`gated` client is provider-agnostic (one HTTP client, two backends), so providers remain swappable. The "escalated call leaves the VM through the gate with disclosure" property is identical either way. |

## Code layout (the build-phase repository)

```
kernel/            # Buildroot external tree: defconfig, kernel fragments, image recipe
image/             # rootfs overlay, weaved as init, verified-boot stubs
crates/
  weaved/          # PID 1: reaper · supervisor · event bus · compositor host
  weave/           # the Weave: renderer, four zones, Materialize/Dissolve
  cortexd/         # Cortex: focus → Intent → Plan (per-event in Phase 1)
  substrated/      # filesystem watcher + indexer (SQLite/FTS/vec)
  modeld/          # llama.cpp + embeddings runtime; facet classification; router
  gated/           # sole network egress + Redaction Gate (Phase-1 stub redactor)
  capd/            # Capability registry, manifest validation, sandboxed workers
  clade-proto/     # shared Cap'n Proto schemas + Rust types
  clade-journal/   # append-only Journal + undo engine (shared lib)
capabilities/      # first-party image manifests (view, adjust, annotate, share-stub, collage)
tools/             # qemu-run.sh, dev harness (run services on host for fast iteration)
```

## Milestones

Each milestone is independently demonstrable. A host-side dev harness (`tools/`) runs the
services on a normal Linux desktop so the mind plane is iterable without rebooting an image —
the image build is exercised in CI and at each integration point, not on every edit.

- **M0 — Scaffold.** Rust workspace, `clade-proto` schemas, `qemu-run.sh`, CI that builds
  the workspace and the Buildroot image. *Demo: `cargo build` + a booting empty image prints a banner.*
- **M1 — Boot to a frame.** Buildroot minimal image; `weaved` as PID 1 (reap + supervise);
  quiet kernel; DRM/KMS via virtio-gpu; one solid frame + the breathing presence dot.
  *Gate: cold boot in QEMU to a Clade frame, no login, no console, < 10s.*
- **M2 — The Weave shell (+ renderer decision).** Four zones laid out; Materialize/Dissolve
  animations; presence states; the Intent Bar accepts text. Prototype wgpu vs. web-engine and
  **lock the choice.** *Demo: the idle Weave from the mockups, running natively.*
- **M3 — Substrate.** `substrated` watches a folder, indexes images into SQLite (metadata +
  FTS + vectors), assigns stable IDs. *Demo: drop 20 photos in, query related-by-embedding from a CLI.*
- **M4 — modeld + Facets.** llama.cpp + embedding models in `modeld`; Facet Resolution returns
  `image` with confidence; the router skeleton emits `{engine, confidence, reason}`.
  *Demo: classify a folder; router explains a local-vs-cloud choice.*
- **M5 — Capabilities + Materializer.** `capd` loads image manifests, sandboxes workers,
  hands the Weave real Surfaces (view, crop/adjust, annotate). Opening a photo materializes
  the halo. *Gate: open a photo → correct Surfaces form → editing works.*
- **M6 — Cortex + Foresight + Journal + the gated call.** `cortexd` forms an Intent on focus;
  Foresight surfaces related content with a *why*; `clade-journal` records the edit and undoes
  it; `gated` runs one escalated image-description call (host Ollama, or Gemini as fallback)
  through the redaction stub with disclosure.
  *Gate: the full exit-gate demo, minus polish.*
- **M7 — Integrate & capture.** Everything in the booted image (not the host harness); record
  the exit-gate run; measure the standing metrics (boot time, materialize p95, cloud bytes).

## Verification strategy

- **Unit/integration in CI** per crate; `clade-journal` undo has property tests (every
  `undoable` event's inverse restores prior state).
- **Golden-path integration test**: a headless harness drives focus→materialize→predict→undo
  against the real services (no display), asserting Journal contents and Foresight output.
- **Boot test in CI**: build the image, boot it in QEMU headless, assert `weaved` reaches the
  "Weave ready" event within budget.
- **The exit-gate demo** is the human acceptance test, recorded.

## Phase-1 risks

| Risk | Mitigation |
|---|---|
| Renderer choice wrong / slow | M2 prototypes both against the mockup animation set before committing. |
| DRM/KMS + virtio-gpu bring-up eats time | Keep M1 to a single frame; wlroots-as-library handles the plumbing; QEMU only. |
| On-device model too heavy for a demo VM | 3–4B class quantized, CPU inference inside the VM (12–16GB budget); measure at M4; the router/`modeld` boundary keeps it swappable. The heavy tier rides the host 5090, not the VM. |
| Redaction correctness | Phase-1 ships a **stub** redactor (regex/entity list) and says so; real gate is Phase 2. Scope is one benign image-description call. |
| Scope creep from the resident-mind vision | Hard line: the Current-across-sessions, Dreamtime, Workbench, Curiosity, Delegation are Phase 2. Phase 1 ships only Journal + per-focus Intent. |

## Owner decisions — all resolved

1. ~~**Product name**~~ — **decided: Clade.**
2. ~~**Phase-1 mode**~~ — **decided: Images.**
3. ~~**Demo VM budget**~~ — **decided:** owner's box is an RTX 5090 + modern AMD CPU + 32GB RAM; the reference VM gets 8 vCPUs / 12–16GB, KVM-accelerated.
4. ~~**Escalation engine**~~ — **decided:** no Claude API in this deployment; default heavy tier is the owner's **Ollama** server (Gemma-27B-class on the 5090), **Gemini API** as opt-in fallback. Provider-agnostic client in `gated`.

---
*This plan realizes Phase 1 of [11-roadmap.md](11-roadmap.md). Phase 2 (the full resident mind
+ all four modes + real hardware) begins once the exit gate holds.*
