# 08 — Data & Knowledge Model

Five durable stores make up the mind plane's state: the **Substrate index**, the
**Context Graph**, the **Memory**, the **Current**, and the **Journal**. All live on the
user's machine, on a data volume separate from the immutable OS image, encrypted at rest.

## The Substrate

The Substrate overlays the base filesystem (btrfs/ext4) — it indexes content in place and
never takes ownership of the bytes. Files remain ordinary files; the Substrate makes them
*mean* something.

```jsonc
// Substrate item
{
  "id": "sub:item/ph_0142",
  "contentHash": "blake3:…",
  "path": "/data/photos/2026-06-28/IMG_0142.jpg",   // real path, untouched
  "facets": [{ "kind": "image", "confidence": 0.98 }],
  "embeddings": { "clip": "vec:…", "text": null },
  "entities": ["ent:person/mom", "ent:event/beach-trip-jun26"],
  "times": { "created": "…", "modified": "…", "lastFocused": "…" },
  "provenance": { "origin": "camera-import", "chain": ["jrn:evt/…"] }
}
```

- **Index = three engines, one store:** metadata (SQLite), vectors (embedded vector index),
  full text (FTS). Incremental and event-driven via filesystem watchers; heavy extraction
  deferred to Dreamtime.
- **Semantic addressing** resolves descriptions to items: *"the invoice from March"* →
  facet `invoice` ∧ time ≈ March ∧ (sender entity if disambiguation needs it). Ambiguity
  returns candidates honestly rather than guessing silently.
- Content addressing by hash makes moves/renames non-events.

## The Context Graph

The connective tissue everything writes and the Cortex/Foresight traverse.

| Element | Types |
|---|---|
| **Nodes** | content item · facet · entity (person / project / place / event) · capability instance · action event |
| **Edges** | `similar-to` (scored) · `part-of` · `authored-by` · `sent-to` · `used-with` · `followed-by` (temporal/causal) · `derived-from` |

Storage: an edge table over SQLite — relational-until-proven-otherwise; an embedded graph
DB is a swap, not a rewrite, if traversal depth ever demands it. Edges carry weights and
timestamps; Dreamtime decays what stops being true.

## The Memory

Four layers, all inspectable through the Workbench, all individually deletable:

| Layer | Contents | Written by |
|---|---|---|
| **Preferences** | Explicit settings + inferred defaults (always distinguished) | User; Dreamtime (inferred, flagged) |
| **Habits** | Capability co-occurrence and sequences, confidence-weighted, decaying | Dreamtime consolidation |
| **Corrections** | Every dismissed/corrected suggestion with its lesson: *not now* / *never* / *wrong tool* | The Weave, at the moment of feedback |
| **Entities** | Personal knowledge: people, projects, places, events | Substrate extraction + user curation |

Rule: **an unexplainable memory is an invalid memory.** Every entry traces to the events
that taught it (via the Journal), which is what makes the Workbench's evidence views
possible and honest.

## The Current

The Cortex's working context, persisted so the mind survives reboot mid-thought:

```jsonc
{
  "threads": [
    { "id": "cur:thr/beach-trip", "salience": 0.9,
      "items": ["sub:item/ph_0142", "…"], "intents": ["int_8f2c"],
      "openQuestions": ["best-of selection pending"] }
  ],
  "recentOutcomes": ["jrn:evt/…"],
  "budget": { "attention": 3 }
}
```

Bounded by construction (salience decay evicts; Dreamtime distills evictions into Memory
rather than dropping them). Sized to what the on-device model can carry economically.

## The Journal

The append-only, OS-level action log — the reversibility guarantee lives here.

```jsonc
// Journal event
{
  "id": "jrn:evt/9d31",
  "actor": "cortex",                       // cortex | user
  "capability": "cap.image.collage",
  "inputs": ["sub:item/ph_0142", "…"],
  "outputs": ["sub:item/co_0007"],
  "reversibility": "undoable",             // undoable | irreversible
  "inverse": { "op": "remove-derived", "target": "sub:item/co_0007" },
  "consent": null,                         // set for gated-irreversible events
  "engine": "local", "at": "…"
}
```

- **Undoable** events record their inverse at write time — undo is a replay, not a hope.
- **Irreversible** events (send, publish, destroy-beyond-recovery) cannot be journaled as
  done until a consent record exists: staged → presented → confirmed → executed. The
  Journal is the *gate*, not just the log.
- The Journal doubles as the audit trail: every cloud escalation, every Capability
  invocation, every Dreamtime conclusion cites the events behind it. One structure gives
  undo, provenance, audit, and Memory-explainability — which is why it is OS-level and
  not a feature of any Surface.

## Retention & the right to forget

- Journal: full fidelity for a rolling window, then Dreamtime compacts to summaries
  (originals of irreversible-consent records are kept).
- "Forget this": deletes the Memory entry *and* re-weights everything derived from it;
  the Workbench shows what changed.
- Wipe-the-mind: Memory, Current, Graph, and Journal are separable from content — the
  machine can be made new without touching a single file.

---
*Next: [09-privacy-trust.md](09-privacy-trust.md) — the ownership guarantees.*
