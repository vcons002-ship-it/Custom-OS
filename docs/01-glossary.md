# 01 — Glossary: The Morph Lexicon

Every document and mockup in this repository uses this vocabulary. The terms are
load-bearing: each names a subsystem or a guarantee, and the design stays coherent
because the words do.

## Core terms

| Term | Definition |
|---|---|
| **Morph** | The operating system itself (provisional working title — see shortlist below). A standalone, bootable, AI-native OS: the "living computer." |
| **The Weave** | The morphing native userland: compositor + shell + UI. Replaces the desktop/window/app model entirely. One continuous canvas that reshapes around whatever holds focus. |
| **The Cortex** | The intent and reasoning engine — the brain. A *resident system service*, not a per-event function. Turns signals (content opened, words typed, ambient context) into an **Intent** and a **Plan**. |
| **Intent** | A structured interpretation of what the user is trying to do right now. The atomic unit the system reasons about. |
| **Plan** | The Cortex's output for an Intent: which Capabilities to materialize, what Foresight to surface, whether cloud reasoning is warranted. |
| **The Substrate** | The semantic, AI-navigated filesystem: a native layer over a conventional base filesystem (btrfs/ext4). Content is addressable by meaning ("the invoice from March") as well as by path. |
| **Facet** | A semantic interpretation of content, with a confidence score. Replaces rigid file types. A `.txt` may carry a *notes* facet, a *to-do* facet, or a *code* facet; a screenshot of code carries both *image* and *code* facets. |
| **Capability** | The unit of "tool": a declarative manifest — I/O schema + reasoning/prompt spec + tool bindings + surface descriptor + permission scope. Not a program. MCP-shaped, so reasoning engines invoke it natively and third parties can publish them. |
| **Surface** | A *materialized* Capability: the transient UI + logic that appears around content when needed and dissolves when not. Surfaces are stateless by default — durable state lives in the Substrate and Memory. |
| **Materialize / Dissolve** | The lifecycle verbs of Surfaces. Nothing launches, nothing quits; Surfaces form and fade. |
| **Foresight** | The prediction engine. Two streams: **related content** (what belongs with what's in focus) and **next actions** (what you usually do from here). |
| **The Memory** | The long-term user model: habits, preferences, corrections, personal entities (people, projects, places). User-inspectable, editable, wipeable. |
| **The Context Graph** | The live knowledge graph joining content, Facets, entities, Capabilities, and actions — the connective tissue Foresight and the Cortex traverse. |

## The resident mind

| Term | Definition |
|---|---|
| **The Current** | The Cortex's persistent short-term working context — a continuous "now" spanning events and sessions. What happened this morning shapes what opening a file means this afternoon. Distilled into Memory during Dreamtime. |
| **Dreamtime** | Idle-cycle consolidation: indexing new content, distilling the Current into Memory, discovering habit patterns, pre-warming likely Surfaces, and self-critiquing missed or rejected predictions. |
| **The Workbench** | The AI's own inspectable workspace: its notes, hypotheses about the user, drafts in progress, and queued questions. The user can open it, read it, and correct it. |
| **Curiosity** | The AI's channel for asking instead of guessing. Uncertainty is a first-class state; suggestion prominence is proportional to confidence. |
| **Delegation** | Handing a whole task to the OS. The Cortex invokes the same Capabilities the user would, under the same permission scopes, with its work visible. One tool substrate, two hands. |
| **The Journal** | The append-only, OS-level action log — every action, the AI's and the human's — powering universal undo. Distinguishes reversible from irreversible actions and gates the latter behind confirmation. |

## The Weave's zones

| Zone | Role |
|---|---|
| **The Stage** | Center of the canvas; holds the content in focus. |
| **The Tool Halo** | Materialized Surfaces orbiting the Stage. |
| **The Foresight Rail** | Related content and suggested next actions, ranked by confidence. |
| **The Intent Bar** | Natural-language input ("ask the computer") plus system status and trust indicators. |

## Trust vocabulary

| Term | Definition |
|---|---|
| **The Privacy Dial** | User-facing policy spectrum: **Airgapped** (local-only) ↔ **Balanced** (auto-escalate non-sensitive work) ↔ **Cloud-boosted**. Overridable per content item or project. |
| **The Redaction Gate** | The local filter every outbound cloud request passes through. Sensitive spans are redacted or the request is held for consent. |
| **Attention budget** | The finite interruption allowance the system spends to proactively surface things. Dismissals shrink it; accepted suggestions grow it. |

## Product name

The subsystem lexicon above is settled. The product name is provisional; **Morph** is the
working title used throughout these documents. Shortlist for the final decision:

| Name | Case for it |
|---|---|
| **Morph** *(recommended)* | Names the defining behavior — the OS that reshapes around you. Short, verb-like, honest. |
| **Aria** | The living, expressive quality; a system with a voice. Softer, more companion-like. |
| **Lumen** | Light/clarity; the calm, legible qualities of the design. |
| **Flux** | Continuous change as identity. Edgier; risks sounding unstable. |
| **Continuum** | The deepest claim — continuity is what makes it alive. Longer, more formal. |

Renaming later is a find-and-replace across `docs/` and `mockups/`; nothing structural
depends on the product name.

---
*Next: [02-system-architecture.md](02-system-architecture.md) — the full stack, bottom to top.*
