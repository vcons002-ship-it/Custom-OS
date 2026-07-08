# Clade — Session Handoff

Living document for picking this project up cold. Last updated after **M3** (the Substrate)
merged to `main`. Read [00-vision.md](00-vision.md) → [01-glossary.md](01-glossary.md) first if
the vocabulary below is unfamiliar.

---

## 1. What Clade is (one paragraph)

**Clade** is a standalone, AI-native operating system — a "living computer." You never open a
program: you open *content*, and the tools you need materialize around it, then dissolve when
you move on. It boots directly into **the Weave** (its native userland) with no desktop, no
windows, no apps; a bare Linux kernel is used *only* as the hardware layer. The mind is
**resident** — it holds context across your day, consolidates while idle, asks when unsure, and
can be handed whole tasks. Name = Greek *klados*, "branch": the first of a new lineage of
computers. (One letter from *Claude*, by coincidence — kept as an homage.)

---

## 2. Where things are (repo geography)

- **`docs/00`–`12`** — the complete design (vision, glossary, system architecture, OS layer,
  the 8 AI subsystems, the resident mind, hybrid AI, interaction model, data/knowledge model,
  privacy/trust, tech stack, roadmap, risks). `docs/phase-1-plan.md` is the M0–M7 engineering plan.
- **`mockups/`** — 11 self-contained clickable HTML screens (`open mockups/index.html`).
- **`crates/`** — the Rust workspace (the actual OS). See §4.
- **`kernel/buildroot-external/`** — the Buildroot recipe that builds the bootable OS image.
- **`tools/`** — `dev-run.sh` (host harness), `qemu-run.sh` (boot the image), `setup.sh` (Linux installer).
- **Windows launchers at repo root** — `setup.bat`, `install-deps.bat`, `run.bat`,
  `build-image.bat`, `doctor.bat`. See §6.

---

## 3. Status: roadmap at a glance

| Phase / Milestone | State |
|---|---|
| **Phase 0 — Concept** (docs + mockups) | ✅ merged |
| **M0 — Scaffold** (weaved PID 1, bus, Journal, service stubs, Buildroot tree, CI) | ✅ merged |
| **M1 — Boot to a frame** (DRM/KMS, presence dot) | ✅ merged (parallel sessions) |
| **M2 — The Weave shell** (zones, Materialize/Dissolve, renderer decision) | ✅ merged (parallel sessions) |
| **M3 — The Substrate** (image indexer + embeddings + search) | ✅ **merged (this session)** |
| **M4 — modeld + Facets** (on-device model, real CLIP embeddings, router) | ⏭️ **next** |
| **M5 — Capabilities materialize** (capd loads manifests → real Surfaces) | ⬜ |
| **M6 — Cortex + Foresight + Journal + gated call** (full exit gate) | ⬜ |
| **M7 — Integrate & capture** (metrics, recorded demo) | ⬜ |
| **Phase 2** — resident mind (Current, Dreamtime, Workbench, Delegation) + all 4 modes + real HW | ⬜ |
| **Phase 3** — third-party Capabilities (MCP), semantic addressing, multi-device | ⬜ |

> Note: parallel sessions also worked this repo; **`main` is the source of truth**. M1/M2 landed
> via other sessions and are already in `main`. Always `git fetch origin main` before assuming state.

---

## 4. The codebase (crates)

Rust workspace; `weaved` is PID 1 and supervises the mind-plane services. Bus = newline-JSON
over a Unix socket (`clade-proto`). Everything durable lives on `/data` (the persistent volume).

| Crate | Role | Depth |
|---|---|---|
| `weaved` | init / supervisor / event-bus host; mounts `/data`; clean SIGTERM | real |
| `clade-proto` | bus `Event` enum + `BusClient` (has `spawn_drain`) | real |
| `clade-journal` | append-only action log + consent-gated irreversible events | real (undo engine is M6) |
| `weave` | the compositor/UI | M1/M2 (check `main`) |
| `substrated` | **the Substrate** — image indexer, embeddings, search (M3) | **real, this session** |
| `modeld` | on-device model runtime | **stub → M4** |
| `cortexd` | the Cortex (intent/plan) | stub → M6 |
| `capd` | Capability registry / materializer | stub → M5 |
| `gated` | sole network egress + Redaction Gate | stub → M6 |

### substrated (M3) internals — what a future session must know
- **Identity = blake3 content hash.** `items` = one row per unique content; `files` = many
  paths → one item (dedup) **and** the change-detection stat cache. `embeddings` is
  **model-tagged** (`phash-hist-v1` now; M4 adds `clip-vit-b32` alongside — no schema change).
- **Embedding is model-free** (`crates/substrated/src/embed/`): 160-dim = DCT structure (DC
  dropped) + 4×4 YCbCr layout + HSV Hellinger histogram + aspect. **Behind the `Embedder`
  trait** — M4 swaps CLIP in here and bumps `db::EMBED_MODEL`, then `reindex`.
- **Single-writer indexer thread** owns the `!Sync` SQLite connection; queries use a warm
  `Arc<RwLock<Vec<CacheEntry>>>` cache + a read-only connection. Brute-force cosine (personal scale).
- **CLI**: `substrated query <path> [--top N] [--time-aware]｜list｜stats｜reindex`. Talks to the
  daemon's control socket if up, else opens the DB directly. Env: `CLADE_SUBSTRATE_DB`,
  `CLADE_LIBRARY`, `CLADE_SUBSTRATE_CTL`, `CLADE_BUS`, `CLADE_SUBSTRATE_DEBOUNCE_MS`.
- **Bus**: adds one event, `SubstrateChanged { sub_id, content_hash, path, change }`
  (Indexed/Updated/Removed), emitted only on live changes (cold scan is silent).

---

## 5. How to build / run / test (Linux or WSL2)

```sh
tools/setup.sh                 # one-time: installs Rust, QEMU, Buildroot deps; verifies dev loop
cargo build --workspace        # build everything
cargo test  --workspace        # 42 tests
cargo clippy --workspace --all-targets   # CI runs this with -D warnings
tools/dev-run.sh               # run the mind plane on the host (no VM) → weave-ready

# The real OS image (first build ~30–60 min, cached after):
cd ../buildroot
make BR2_EXTERNAL="$PWD/../Custom-OS/kernel/buildroot-external" clade_x86_64_defconfig
make
../Custom-OS/tools/qemu-run.sh output/images   # boot Clade in QEMU (auto-creates /data volume)

# substrated demo (standalone, no daemon):
CLADE_LIBRARY=~/photos CLADE_SUBSTRATE_DB=/tmp/s.db CLADE_SUBSTRATE_CTL=/none \
  target/debug/substrated reindex && ... substrated query ~/photos/a.jpg --top 5
```

**Quality gate before any commit:** `cargo fmt --all --check` · `cargo clippy --workspace
--all-targets` (must be 0 warnings) · `cargo test --workspace` · `cargo build --workspace --locked`
(the committed `Cargo.lock` is what the Buildroot image build needs — regenerate + commit it
whenever deps change, or the image build fails).

---

## 6. The owner's environment (important operational context)

- **Machine:** Windows 11, RTX 5090, modern AMD CPU, 32GB RAM. Development happens in **WSL2
  (Ubuntu-24.04)**; the OS itself runs in **QEMU/KVM** (nested virt via `.wslconfig`).
- **Owner workflow:** `setup.bat` once → `build-image.bat` once → `run.bat` every time.
  `install-deps.bat` only when deps change. `doctor.bat` diagnoses WSL hangs.
- **Escalation ("cloud") tier:** no Claude API available. Default heavy engine is the owner's
  **Ollama server (Gemma-27B-class on the 5090)**; **Gemini API** is the opt-in fallback. The
  router/`gated` client is provider-agnostic. This matters for **M6**.
- **The Windows↔WSL launchers were a real battleground** (quoting, line endings, ownership,
  WSL service hangs). Rules encoded in the `.bat` files, do not regress: use `wsl --cd "%~dp0."`
  (trailing dot!), no `wslpath`/`$( )`/escaped quotes in payloads, `.gitattributes` forces LF for
  scripts + CRLF for `.bat`, and the payloads `sed`-strip CRs after syncing into WSL.

---

## 7. Decisions locked (don't re-litigate)

- **Its own OS**, not an app on a host: bare Linux kernel = hardware layer only; the Weave is PID 1.
- **Hybrid AI**: on-device model routes/embeds; escalation tier (owner's GPU / cloud) for heavy work.
- **Fully radical UI**: one morphing canvas, no apps/windows.
- **Local-first / owned**: Privacy Dial, redaction gate on all egress, per-Capability permissions,
  universal-undo Journal.
- **Resident mind** (the differentiator): the Current, Dreamtime, Workbench, Curiosity, Delegation.
- **Product name: Clade.** Content modes (conceptual): images, PDFs, code, notes. **Phase-1 mode: images.**
- **Tech**: Buildroot (2025.02.x LTS) · Rust everywhere · SQLite bundled (the only sanctioned C dep) ·
  MCP-shaped Capabilities · brute-force vectors at personal scale.

---

## 8. Working method that has served well (recommended)

1. **Plan mode + Explore/Plan agents** for scoping; **judged design panel** (3 independent
   designs → synthesize) before writing a large subsystem — this produced M3's excellent spec.
2. **Implement inline, compile in stages, test as you go.** Prove the exit gate with an
   end-to-end test *and* a live CLI/booted-image run, not just unit tests.
3. **Adversarial review workflow** (find per-dimension → verify each) over any large diff before
   merge. On M3 it caught two high-severity bugs the 38 passing tests missed. Worth the tokens.
4. **Per-milestone: PR → merge to `main` → reset the working branch onto `origin/main`.** The
   owner wants each milestone merged. Working branch is `claude/ai-adaptive-os-design-cu824a`.
5. **Commit message trailers** the owner's tooling expects:
   `Co-Authored-By: Claude ...` and `Claude-Session: https://claude.ai/code/session_...`.
   Never put the model identifier in commits/PRs.

---

## 9. M4 — the next milestone (concrete starting point)

**Goal:** `modeld` becomes a real on-device model runtime; Facet Resolution + the local/escalate
router come online; real CLIP-class embeddings replace `phash-hist-v1` behind the existing `Embedder`
trait.

Concrete first steps:
- Link **llama.cpp** (bundled, cross-compile-friendly — mirror how `rusqlite` bundled was vetted)
  + a compact **CLIP-class image embedder** into `modeld`.
- Implement a `ClipEmbedder: Embedder` in/alongside `substrated::embed`; models coexist by
  `model` tag, so run both, then `reindex --embeddings` to migrate. Bump `db::EMBED_MODEL`.
- **Facet Resolution**: cheap deterministic (magic bytes/extension) → local-model classification
  → escalate only on low confidence. Emit facets with confidence.
- **Router** emits `{engine, confidence, reason}` (reason is user-visible). Wire `gated`'s
  provider-agnostic client to the owner's Ollama first, Gemini fallback (see §6).
- **De-risk first**, as with M3: confirm the model runtime cross-compiles in Buildroot and that
  the in-VM footprint (3–4B / CPU) is acceptable *before* building the milestone.

**M4 exit gate** (from the roadmap): classify a folder; the router explains a local-vs-escalate
choice for a given item.

---

## 10. Known risks / watch-items carried forward

- **Buildroot image cross-compile** is the least-exercised link (needs a real multi-GB build on
  the owner's machine; can't run in the assistant's container). Any new C-touching dep (M4's model
  runtime) must be verified there, and `Cargo.lock` kept committed.
- **`panic=abort`** in release: a panic in any service crash-loops it under weaved. Keep decode/
  model paths `Result`-clean with poison-quarantine (M3 pattern).
- **Parallel sessions** touch this repo — always reconcile with `origin/main`.
- Full risk register: [12-risks-open-questions.md](12-risks-open-questions.md).

---

*Pointers: design starts at [00-vision.md](00-vision.md); build/roadmap at
[11-roadmap.md](11-roadmap.md) and [phase-1-plan.md](phase-1-plan.md); this session's code lives in
`crates/substrated/`.*
