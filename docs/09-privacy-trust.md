# 09 — Privacy, Security & Trust: An OS You Own

A living computer knows you. That is only acceptable if the knowing is **yours**: stored
on your machine, readable by you, correctable by you, deletable by you — and if every
time knowledge moves or acts, you can see why. Morph's trust model is not a settings
page; it is load-bearing architecture.

## Principles

1. **Local-first by default.** Content, Memory, the Current, the Graph, the Journal —
   all on-device. The cloud is an invited guest, per-operation, by policy.
2. **Nothing leaves without passing the Redaction Gate.** All egress flows through
   `gated` — the only service with a network path ([03-os-layer.md](03-os-layer.md)).
3. **Legibility over reassurance.** Every suggestion, escalation, and learned habit shows
   its *why*. Trust is built from a thousand small honest disclosures, not a privacy policy.
4. **Least privilege, declared.** Capabilities state their scopes in their manifests and
   get exactly those, sandboxed. No ambient authority anywhere above the kernel.
5. **Reversibility as a guarantee.** The Journal makes every action undoable or explicitly
   gated ([08-data-knowledge-model.md](08-data-knowledge-model.md)).

## The Privacy Dial

One user-facing policy control, with per-item and per-project overrides:

| Position | Behavior |
|---|---|
| **Airgapped** | Nothing leaves the device, ever. Cloud-class work queues or declines, with its reason shown. The OS remains fully a computer. |
| **Balanced** *(default)* | Non-sensitive work may auto-escalate to cloud; anything the gate flags as sensitive requires explicit consent per operation. |
| **Cloud-boosted** | Escalation is liberal; the gate still redacts, and irreversible actions still stop. |

Overrides bind to Substrate items and entities: *this folder never leaves; this project
is airgapped; anything involving this person asks first.*

## The Redaction Gate

Every outbound request is assembled, then filtered on-device:

1. **Minimize:** the smallest sufficient slice — excerpt not file, summary not corpus,
   embedding not text.
2. **Detect:** local PII/sensitivity pass (names, credentials, financial identifiers,
   flagged entities).
3. **Redact or hold:** mask sensitive spans, or hold the request for consent when masking
   would gut it.
4. **Disclose:** the requesting Surface shows what left, what was masked, and the router's
   reason — before dispatch for held requests, in the Journal always.

Honesty about limits: detection is probabilistic. False negatives are the residual risk;
the dial, per-entity overrides, and minimization-by-default exist to bound the blast
radius, and [12-risks-open-questions.md](12-risks-open-questions.md) carries this as an
open hard problem rather than a solved one.

## Capability permissions

- Manifests declare scopes (`read:selected-items`, `write:substrate`, `net:send-email`, …);
  `capd` enforces them in sandboxed workers — namespaces, seccomp, no filesystem beyond
  the granted mounts, no network unless scoped *through `gated`*.
- **Consent is per-Capability, at first materialization**, in plain language ("Collage
  can read the items you select and save new images. Nothing else."). Scope changes
  re-prompt. No install-time walls of permissions, because there is no install.
- Fighting consent fatigue: scopes are few and coarse enough to mean something; identical
  repeat grants stay granted; *acting* is consented once per Capability while
  *irreversible acting* is confirmed per event. Different risks, different ceremonies.

## The audit surface

The Journal is the single audit trail: every action, every escalation (with payload
summary and redactions), every Dreamtime conclusion, every Memory write with the events
that taught it. The Weave renders it as a human timeline — *what did you do while I was
away? what have you learned about me this week? what has left this machine today?* —
each entry expanding to its evidence.

## Memory ownership

- **Inspectable:** the Workbench shows hypotheses and Memory with evidence and confidence.
- **Correctable:** confirm / correct / delete are first-class writes with immediate effect
  on prediction; the Workbench honesty constraint applies — what's shown is what's used.
- **Wipeable:** per-entry, per-entity, per-layer, or the whole mind — content untouched.
- **Exportable:** Memory, Graph, and Journal export to a documented open format. Owning
  your computer's model of you includes taking it with you.

## System integrity

Verified boot chain: firmware → signed bootloader → signed immutable OS image (A/B slots,
automatic rollback). Data volume encrypted at rest, unlocked at boot. Mind-plane services
mutually isolated; a compromised Capability worker holds only its declared scopes; a
compromised Surface holds nothing durable at all.

---
*Next: [10-tech-stack.md](10-tech-stack.md) — what the build phase reaches for.*
