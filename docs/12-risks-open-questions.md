# 12 — Risks, Hard Problems, Open Questions

Stated honestly. These are flagged, not solved; several are the actual research content
of Phases 1–2. Each lists the mitigation the design already carries.

## Hard problems

| # | Problem | Why it's hard | What the design already does |
|---|---|---|---|
| 1 | **Intent inference vs. calm** — wrong guesses are clutter; timid guesses kill the magic. *The make-or-break UX risk.* | Prediction quality has a ceiling; annoyance compounds faster than delight | Confidence-proportional prominence; the attention budget; three-way dismissal lessons; Curiosity asking instead of guessing; accept-rate as a standing measure |
| 2 | **Latency** — "tools simply exist" tolerates ~250ms; cloud reasoning takes seconds | Physics + economics | Local-first routing; Foresight pre-warming; Dreamtime preparation; honest progress for cloud work (never a spinner pretending to be local) |
| 3 | **Bounding the Current** — a persistent mind with a finite context window | Salience scoring is guesswork; evict wrongly and the "continuity" promise quietly breaks | Thread structure + decay; Dreamtime distillation so evictions become Memory, not loss; sized to the on-device model |
| 4 | **Dreamtime's power budget** — consolidation vs. battery/thermals | Laptops sleep; the mind wants the night shift | Charge-aware scheduling; interruptible <1s; degraded "catch-up" mode after skipped nights |
| 5 | **Undo at the edge of the world** — a sent email cannot be unsent | Reversibility is an OS guarantee only inside the OS | `reversibility` declared per Capability; the Journal *gates* irreversible events behind staged consent; Delegation stops at one-way doors |
| 6 | **Redaction reliability** — the privacy promise rests on a probabilistic filter | False negatives leak; false positives gut the request | Minimization *before* detection; the dial bounds exposure; per-entity overrides; full egress audit. Residual risk is acknowledged, not waved away |
| 7 | **The "no apps" leap** — decades of window/app muscle memory | Radical is the brief, but learnability decides adoption | Content-first is *more* direct, not less; the Intent Bar as universal fallback ("just say it"); the maintenance console stays a service door |
| 8 | **Consent fatigue vs. least privilege** | Too many prompts train blind clicking | Few, coarse, meaningful scopes; consent at first materialization; only irreversible *events* re-confirm |
| 9 | **Workbench honesty** — inspectability must reflect the actual model, not a story | Neural habit signals resist faithful summarization | Hypotheses trace to Journal evidence; "unexplainable memory is invalid memory" as an architectural rule; deletion provably stops conditioning |
| 10 | **PID 1 across real hardware** — GPUs, suspend/resume, thermal quirks | The classic bring-up tarpit | QEMU as Phase-1 reference; exactly one blessed laptop in Phase 2; the kernel carries the driver burden by design |
| 11 | **Cloud cost** — a chatty router bankrupts the owner | Per-token economics vs. an always-on mind | Local-first as architecture; caching; Dreamtime batching; egress and call counts as standing measures |
| 12 | **Evaluating "it predicted what I needed"** | The core promise resists a benchmark | The standing measures in [11-roadmap.md](11-roadmap.md); Phase-2's week-of-use gate is the honest test |

## Open questions for the owner

1. **Product name** — pick from the shortlist in [01-glossary.md](01-glossary.md) (docs
   currently use **Morph** as the working title).
2. **Phase-1 content mode** — recommendation is **Images**; confirm or override.
3. **On-device model footprint** — a 3–8B model implies real RAM/disk on target hardware;
   how much machine is the floor?
4. **Phase-2 hardware** — which laptop gets blessed for bring-up?
5. **Delegation ambition in Phase 2** — multi-step tasks like the collage flow, or wider
   (email triage, file organization) once trust is earned?
6. **Multi-user future** — v1 is deliberately single-user ("the machine *is* the person's
   computer"). Is shared-machine support ever in scope? It changes Memory, the Current,
   and the trust model materially if so.

## Standing risks accepted knowingly

- **The Linux kernel dependency** is a pragmatic marriage, not a love match; the narrow
  interface ([03-os-layer.md](03-os-layer.md)) keeps annulment possible.
- **Frontier-model dependency for the hardest reasoning** — the router and manifest
  abstraction keep providers swappable, but Phase-2 magic partially rides on cloud model
  quality the project doesn't control.
- **This is a research-grade UX bet.** The mockups make it feel inevitable; only
  Phase 2's week-of-daily-use gate proves it.

---
*End of the design set. Begin again at [00-vision.md](00-vision.md).*
