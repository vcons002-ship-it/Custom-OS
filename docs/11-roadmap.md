# 11 — Roadmap: Concept → Bootable Seed → Resident Mind → Ecosystem

Each phase has an explicit **exit gate** — a demonstrable claim, not a date. A phase is
done when its gate holds, and the gate of each phase de-risks the biggest open question
of the next.

## Phase 0 — Concept *(this repository)*

**Deliverables:** the thirteen design documents and the clickable mockup set.

**Exit gate:** the owner validates the vision, the vocabulary, the "its own OS" stack,
the resident-mind design, and the five flows — by reading the docs and clicking the
mockups — and picks the product name and Phase-1 content mode.

## Phase 1 — Bootable Seed *(QEMU)*

The smallest thing that is honestly Morph: boots as its own OS, has a resident (if
modest) mind, and morphs around one content mode.

**Scope:**
- Buildroot image; `weaved` as PID 1; DRM/KMS rendering in QEMU/KVM (virtio-gpu).
- The Weave with all four zones and real Materialize/Dissolve animation.
- `modeld` with on-device LLM + embeddings; `substrated` indexing a real folder tree;
  deterministic + local-model Facet Resolution.
- **One content mode** — recommendation: **Images** (visually undeniable, tolerant of
  imperfect inference: a wrong related-photo is shrugged off; a wrong code edit is not).
- A handful of first-party Capabilities (view, adjust, annotate, share-stub, collage).
- Foresight: related-content stream only. The Journal with working undo.
- Cloud reasoning behind the Privacy Dial + Redaction Gate for one flow (e.g. image
  description), to prove the full hybrid path end to end.
- The wgpu-vs-web-engine decision resolved by prototype ([10-tech-stack.md](10-tech-stack.md)).

**Exit gate:** *cold boot in QEMU to the idle Weave; open a real photo; correct Surfaces
materialize; one honest related-content suggestion appears with its why; an edit is made
and undone from the Journal; one consented cloud call round-trips through the gate.*

## Phase 2 — The Resident Mind + All Four Modes

**Scope:**
- The Current (persistent, bounded, reboot-surviving) and `dreamd` running the full
  Dreamtime cycle: consolidation, discovery, pre-warming, self-critique.
- The Workbench, Curiosity questions, and the attention economy with three-way
  dismissal lessons feeding Memory.
- Foresight next-actions from habit statistics; confidence-proportional presentation.
- Delegation end-to-end with per-step journaling and gated-irreversible confirmation.
- PDF, code, and notes modes; multi-facet content (screenshot-of-code).
- Real-hardware bring-up: one well-supported x86 laptop.

**Exit gate:** *a week of daily use on real hardware where (a) a habit the user never
stated is learned during Dreamtime, surfaced on the Workbench with evidence, and confirmed;
(b) a delegated task runs visibly and stops at its irreversible step; (c) suggestion
accept-rate and interruption counts trend the right way across the week.*

## Phase 3 — Ecosystem & Beyond

**Scope:**
- Third-party Capabilities: MCP-based publishing, manifest validation, sandbox hardening,
  a signing/review story.
- Semantic addressing matured ("the invoice from March" as a universal address bar).
- Cross-content synthesis; communication modes (the email example from the vision).
- Multi-device: privacy-preserving Memory/Current sync — the mind follows you.
- Yocto revisit if hardware variants proliferate; offline maturity; performance hardening.

**Exit gate:** *a Capability written by someone outside the project installs from a
manifest, runs sandboxed with declared scopes only, and materializes indistinguishably
from first-party tools.*

## Standing measures (every phase)

| Measure | Why it's watched |
|---|---|
| Suggestion accept / dismiss ratio (and the three-way lesson mix) | The make-or-break UX risk, quantified |
| Interruptions per hour vs. attention budget | Calm is a mechanism; verify it |
| Cold boot → interactive Weave time | The "it's really an OS" promise |
| Materialize latency (p95) | The "tools simply exist" promise |
| Cloud calls per day + bytes egressed | Local-first, verified not asserted |

---
*Next: [12-risks-open-questions.md](12-risks-open-questions.md) — what could sink this.*
