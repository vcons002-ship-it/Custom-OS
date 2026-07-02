# Phase 1 — The Bootable Seed (engineering plan)

The smallest artifact that is honestly this OS: it boots as its own operating system,
has a resident (if modest) mind, and morphs around one content mode. Everything here
serves the single **exit gate** below; anything not on that path is Phase 2.

## Exit gate (the demo that ends Phase 1)

> Cold-boot the image in QEMU to the idle Weave. Open a real photo from a watched folder.
> The correct tool Surfaces materialize around it. One honest related-content suggestion
> appears in the Foresight Rail with an expandable *why*. Make an edit and undo it from
> the Journal. Trigger one cloud call (image description) and watch it round-trip through
> the Redaction Gate with its reason shown.

Six capabilities, end to end: **boot · morph · index · predict · undo · one gated cloud call.**

## Scope

- **One content mode: Images.** Chosen to de-risk: a wrong related-photo is shrugged off;
  a wrong code edit is not. Visually undeniable in a demo.
- **One machine: QEMU/KVM** with virtio-gpu. No real-hardware bring-up (that is a Phase 2 gate).
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
| Weave renderer | **Prototype both in M2, decide at M2 exit**: `wgpu` retained scene graph vs. an embedded web engine (WebKitGTK/Servo) reusing the mockup HTML/CSS | This is the roadmap's named deferred decision. Criteria: animation fidelity, iteration speed, memory, input latency. Ship whichever clears the bar; the other becomes the fallback. |
| Inter-service IPC | **Cap'n Proto RPC over Unix domain sockets** | Schema-first messages shared via the `clade-proto` crate; fast, typed, no broker. |
| Event bus | A small pub/sub in `weaved` over the same sockets | Focus events, materialize/dissolve, journal appends. |
| On-device LLM | A 3–4B-class instruction model (GGUF) via **llama.cpp** linked into `modeld` | Exact model chosen at M4 against current benchmarks; swappable by design. |
| Embeddings | A compact CLIP-class image model + small text embedder in `modeld` | Powers Substrate vectors + Foresight similarity. |
| Stores | **SQLite**: metadata + Context Graph (edge table) + FTS5, **sqlite-vec** for vectors; Journal as append-only WAL DB | Embedded, one backup story. |
| Cloud | **Claude API** via `gated` for the single image-description call | Model id/pricing verified against the live `claude-api` reference at M6. |

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
  it; `gated` runs one Claude image-description call through the redaction stub with disclosure.
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
| On-device model too heavy for a demo VM | 3–4B class; measure at M4; the router/`modeld` boundary keeps it swappable. |
| Redaction correctness | Phase-1 ships a **stub** redactor (regex/entity list) and says so; real gate is Phase 2. Scope is one benign image-description call. |
| Scope creep from the resident-mind vision | Hard line: the Current-across-sessions, Dreamtime, Workbench, Curiosity, Delegation are Phase 2. Phase 1 ships only Journal + per-focus Intent. |

## Decisions needed from the owner before M4/M6

1. ~~**Product name**~~ — **decided: Clade.**
2. **Confirm Images as the Phase-1 mode** (recommended) or override.
3. **Demo VM budget** — how much RAM/vCPU the reference QEMU machine may assume (drives model size at M4).
4. **Claude API access** for the single M6 cloud call — key/proxy availability in the dev environment.

---
*This plan realizes Phase 1 of [11-roadmap.md](11-roadmap.md). Phase 2 (the full resident mind
+ all four modes + real hardware) begins once the exit gate holds.*
