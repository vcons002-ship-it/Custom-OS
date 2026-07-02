# 06 — Hybrid AI: The Router, the Manifest, the Lifecycle

Clade's mind is hybrid: an on-device model baked into the OS image for the fast, private,
always-available work, and cloud reasoning (the Claude API) for the genuinely hard work.
This document specifies the split, the router that enforces it, the Capability manifest
(what a "tool" actually is), and the Materialize/Dissolve lifecycle.

## The split

| On-device (`modeld`, always available) | Cloud (Claude API via `gated`, by policy) |
|---|---|
| Intent first-pass routing and classification | Ambiguous or multi-step intent planning |
| Facet Resolution (common cases) | Deep content understanding ("explain this codebase") |
| Embeddings for the Substrate and Foresight | Cross-content synthesis ("summarize these 20 PDFs") |
| Habit statistics, graph traversal, related-content ranking | Novel Capability composition / delegated task planning |
| The Redaction Gate (PII/sensitivity detection) | Natural-language authoring, rewriting, transformation |
| The Current's upkeep; Dreamtime consolidation | Dreamtime's hardest pattern-mining (opt-in, redacted) |

The on-device tier is a small instruction-tuned LLM (3–8B class) plus a compact embedding
model, shipped *inside the OS image* — the mind is present before any network exists.
Model choices and runtimes in [10-tech-stack.md](10-tech-stack.md).

## The router

Every Plan that might leave the device passes a local policy decision:

```jsonc
{
  "engine": "cloud",                  // local | cloud
  "reason": "multi-document synthesis exceeds local capability",
  "confidence": 0.91,                 // local tier's confidence in its own interpretation
  "sensitivity": "low",               // redaction gate's assessment
  "redactions": ["acct# → ▮▮▮▮"],     // what will be masked before sending
  "budget": { "latencyMs": 4000, "class": "interactive" }
}
```

Inputs to the decision: local confidence (low → escalate), complexity signals (multi-step,
multi-item, generative), sensitivity (private content prefers local; if cloud is needed,
redact or ask), connectivity and cost budget, and the user's **Privacy Dial** position
([09-privacy-trust.md](09-privacy-trust.md)). The `reason` string is not internal — the
Weave surfaces it, because a legible router is the difference between "hybrid" and "leaky."

**Data minimization is structural:** the smallest sufficient slice goes out — an excerpt,
not the file; a summary, not the corpus; embeddings where raw text isn't needed. All egress
passes through `gated`; no other service has a network path.

## What a Capability is

A Capability is a **declarative manifest, not a program**:

```jsonc
{
  "id": "cap.image.collage",
  "version": "1.2.0",
  "appliesTo": [{ "facet": "image", "min": 0.7, "count": ">=2" }],
  "intentTriggers": ["combine images", "collage", "montage"],
  "contract": {
    "inputs":  { "items": "sub:item[]", "layout": "enum?" },
    "outputs": { "result": "sub:item" },
    "tools": [                              // MCP-shaped tool specs
      { "name": "compose_grid",  "bind": "worker:image-compose" },
      { "name": "suggest_layout","bind": "reasoning" }
    ],
    "reasoning": "prompt-template:collage-layout"   // used when a step needs a model
  },
  "surface": {
    "form": "halo-panel",                   // how it materializes in the Weave
    "controls": ["layout-picker", "spacing", "accept"],
    "stage": "preview-overlay"
  },
  "permissions": ["read:selected-items", "write:substrate"],
  "memoryKeys": ["layout-preference"],
  "reversibility": "undoable"               // undoable | gated-irreversible
}
```

Five consequences fall out of this shape:

1. **Declarative ⇒ reasonable-about.** The Cortex can read, rank, and compose manifests.
2. **MCP-shaped ⇒ natively invocable.** Tool specs map directly onto the Model Context
   Protocol, so cloud reasoning drives Capability tools without adaptation, and a
   third-party ecosystem has a standard to publish against.
3. **One substrate, two hands.** The same manifest serves the user (as a Surface) and the
   Cortex (as Delegation steps) under the same permission scopes.
4. **Sandboxed by construction.** Tool bindings run as `capd` workers with exactly the
   declared scopes; a Capability has no ambient authority ([03-os-layer.md](03-os-layer.md)).
5. **Reversibility is declared**, so the Journal knows what to gate before anything runs.

## The Materialize / Dissolve lifecycle

```
eligible ──selected──▶ materialized ──▶ active ──idle/blur──▶ dissolved
   ▲        (Plan)      (Surface formed,                        (UI gone;
   │                     tools bound,                            state was
   └── facets match      permissions checked)                    never here)
```

- **Eligible:** Facet predicates match the focused content. Cheap; computed continuously.
- **Selected:** the Plan names it (or the user asks for it via the Intent Bar).
- **Materialized:** `capd` binds tools in a sandbox; the Weave animates the Surface in.
  Foresight may pre-warm to this state *hidden* for instant feel.
- **Active:** in use. Every tool invocation appends to the Journal.
- **Dissolved:** focus lost or idle timeout; the Surface unrenders and workers stop.
  Nothing is saved at dissolve time because nothing durable ever lived in the Surface.

## Degraded modes, stated plainly

| Condition | Behavior |
|---|---|
| Offline / Airgapped dial | Local-only: everything works except cloud-class reasoning, which queues or declines with its reason shown. |
| `modeld` down | Deterministic facets, no Foresight, no Curiosity; the Weave says the mind is resting, not pretends otherwise. |
| Cloud degraded/slow | Router reroutes locally where possible; otherwise the Surface shows honest progress, never a spinner pretending to be local. |

---
*Next: [07-interaction-model.md](07-interaction-model.md) — the Weave, and the flows.*
