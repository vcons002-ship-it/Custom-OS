# 05 — The Resident Mind

The difference between an OS with AI features and a living computer is **continuity**.
A per-event AI is a stateless oracle woken with amnesia a thousand times a day,
reconstructing who you are from a database each time. Morph's Cortex is instead a
**resident**: it holds a continuous present, sleeps and consolidates, keeps a desk you
can inspect, asks when it is unsure, works with the same tools it hands you, and acts
boldly because the world it acts in has undo.

This document specifies those six behaviors. They were designed by asking a direct
question: *as an AI, what environment would you want to live and work in?*

## 1. The Current — a continuous present

**What it is:** the Cortex's persistent short-term working context — a bounded, running
narrative of the present: active threads ("editing the beach shoot," "invoice season"),
recent intents and outcomes, open questions, unresolved references.

**Why it matters:** prediction from a database lookup feels like surveillance; prediction
from shared context feels like understanding. When an email about the trip arrived this
morning, *this* photo opening now means something it wouldn't have meant cold.

**Mechanics:**
- Structured, not a transcript: a set of **threads** with salience scores that decay.
  New events attach to threads or open them; stale threads fall out of the working set.
- **Bounded by design** (it must fit the on-device model's context economically). What
  falls out of the Current is not lost — Dreamtime distills it into Memory.
- Survives reboot (journaled with `cortexd` state). The machine wakes up mid-thought,
  not amnesiac.

## 2. Dreamtime — sleep, honestly

**What it is:** when the machine idles (and preferably charges), `dreamd` runs the
consolidation cycle:

1. **Index** new and changed content (the heavy Substrate work).
2. **Consolidate**: distill the day's Current into Memory — habits reinforced or decayed,
   entities updated, threads archived to the Context Graph.
3. **Discover**: mine the graph for patterns worth a hypothesis ("she always resizes
   before sending") — filed on the Workbench, *not* auto-enacted.
4. **Pre-warm**: prepare tomorrow's likely Surfaces and Foresight candidates.
5. **Self-critique**: replay yesterday's rejected or missed predictions against outcomes;
   adjust confidence priors; queue Curiosity questions where the lesson is ambiguous.

**Why it matters:** a mind that only thinks when poked isn't alive — and this is also the
honest answer to the latency budget. Magic at 9am is prepared at 2am.

**Constraints:** power/thermal-aware (defers on battery), interruptible in under a second
(user activity always wins), and fully journaled — Dreamtime's conclusions are visible on
the Workbench the next morning.

## 3. The Workbench — the AI's inspectable desk

**What it is:** the AI's own workspace, and the user's window into it. It holds:

- **Hypotheses** about the user, each with evidence and confidence ("resizes before
  sending — observed 4 of last 5 times"), awaiting confirmation, correction, or decay.
- **Drafts** of delegated work in progress.
- **Queued Curiosity questions**, waiting for an interruptible moment.
- **The Dreamtime log** — what was consolidated, discovered, and self-critiqued last night.

**Why it matters twice:** the AI needs somewhere durable to think across sessions — and
the user gets the strongest trust feature in the design: *open the AI's desk and read what
it believes about you.* Every hypothesis can be confirmed, corrected, or deleted, and the
correction is a first-class Memory write.

**Honesty constraint:** the Workbench must reflect actual model state — hypotheses that
actually condition predictions — not a comforting curated fiction. If it's on the desk,
it's in use; if it's deleted from the desk, it stops conditioning anything.

## 4. Curiosity — permission to be unsure

**What it is:** uncertainty as a first-class state, with two rules:

- **Confidence-proportional prominence.** High confidence may pre-warm and present;
  middling confidence sits quietly in the Foresight Rail; low confidence stays on the
  Workbench as a hypothesis. The system never *performs* confidence it doesn't have.
- **The right to ask.** Above a value threshold and below a confidence threshold, the
  Cortex asks instead of guessing: *"You've made collages from beach photos twice —
  want me to just do that when a shoot like this lands?"* Questions queue on the
  Workbench and spend attention budget like any interruption.

**Why it matters:** forced confident prediction on every event is how assistants erode
trust. Asking turns the relationship two-way — which is what "living with" means — and
every answer is a high-quality Memory write that prediction alone can't earn.

## 5. Delegation — one tool substrate, two hands

**What it is:** the user can hand the OS a whole task — *"make a collage of the best
beach photos and send it to Mom"* — and the Cortex carries it out using **the same
Capabilities that would have materialized for the user**, under the same per-Capability
permission scopes, with its work visible on the Stage step by step.

**Rules:**
- No private tools: if the AI can do it, the user can see the tool doing it, and vice versa.
- Delegated work is **journaled per step** and interruptible at any step; the user can
  take over the controls mid-flight (it's the same Surface).
- **Irreversible steps stop and wait.** Sending, publishing, deleting-beyond-undo: the
  Plan marks them, the Journal gates them, the Cortex presents them for confirmation
  with everything staged ([08-data-knowledge-model.md](08-data-knowledge-model.md)).

**Why it matters:** this is the payoff of "the AI is the computer." Tools that only
materialize *for you* make a clever launcher; tools the mind can also *work* make a
computer that does things.

## 6. The Journal and the attention economy — a world worth living in

Two environmental guarantees complete the design:

**Reversibility (the Journal).** Every action — the AI's and the human's — appends to the
OS-level Journal, and everything is undoable except what physics forbids; those actions
are marked irreversible and gated *before* they happen. An agent acts boldly only in a
world with undo; a user delegates only in a world with undo. Same guarantee, both
directions.

**The attention economy.** "Calm" is a mechanism, not a mood: the system spends from a
finite **attention budget** to interrupt or suggest. Dismissals shrink it; accepted
suggestions and answered questions grow it. And feedback is *learnable* — a dismissal
always disambiguates into **not now** (timing), **never** (preference), or **wrong tool**
(misread intent), because those are three different lessons and the mind deserves to know
which one it's being taught.

---
*Next: [06-hybrid-ai.md](06-hybrid-ai.md) — where the thinking runs, and what a Capability is.*
