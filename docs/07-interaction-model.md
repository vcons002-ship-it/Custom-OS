# 07 — Interaction Model: The Weave

One continuous canvas. No apps, no windows, no launching, no quitting. The environment
morphs around whatever holds focus. This document specifies the zones, the interaction
rules, and the five end-to-end flows the mockups demonstrate.

## The four zones

```
┌────────────────────────────────────────────────────────────┐
│                                              ╭───────────╮ │
│                                              │ FORESIGHT │ │
│        ╭─ TOOL HALO ─╮                       │   RAIL    │ │
│     ╭──┤  Surfaces   ├──╮                    │ related   │ │
│     │  ╰─────────────╯  │                    │ content   │ │
│     │      STAGE        │                    │ ───────── │ │
│     │  (focused content)│                    │ next      │ │
│     ╰───────────────────╯                    │ actions   │ │
│                                              ╰───────────╯ │
│   ● presence      ╭──────────────────────╮                 │
│                   │      INTENT BAR      │                 │
│                   ╰──────────────────────╯                 │
└────────────────────────────────────────────────────────────┘
```

| Zone | Behavior |
|---|---|
| **Stage** | Holds the content in focus — one item, a selection, or a delegated task's live progress. Content is never inside a "viewer"; it is simply *present*, at the center. |
| **Tool Halo** | Materialized Surfaces orbit the Stage, grouped by verb (understand · change · connect). Idle Surfaces dissolve; the halo never crowds. |
| **Foresight Rail** | Related content above, next actions below, each with confidence-proportional prominence and an expandable *why*. Dismissals ask which lesson: *not now* / *never* / *wrong tool*. |
| **Intent Bar** | Natural language in ("ask the computer"), status out: the presence indicator (idle · thinking · asking · dreaming), the router's local/cloud disclosure, and the Privacy Dial position. |

**The presence indicator** is the mind made visible: a small breathing mark that shows
state honestly — including "waking" during boot and "resting" if `modeld` is down.

## Interaction rules

1. **Nothing launches.** Focus changes; the environment reshapes. Materialize ≈ 250ms,
   Dissolve ≈ 180ms — fast enough to feel immediate, slow enough to read as *forming*.
2. **Confidence is visible.** High-confidence suggestions may present themselves;
   middling ones sit quietly in the Rail; low-confidence hypotheses stay on the
   Workbench. The system never performs certainty it doesn't have.
3. **Every why is one gesture away.** Suggestions, materialized tools, cloud escalations —
   all expose their reason on hover/press.
4. **Every action is one gesture from undone.** The Journal is reachable from anywhere;
   irreversible steps present a staged confirmation *before* they run.
5. **Keyboard-first, language-always.** The Intent Bar is always a keystroke away; anything
   the halo offers can also be asked for in words.
6. **Interruptions spend budget.** Proactive surfacing draws from the attention economy
   ([05-resident-mind.md](05-resident-mind.md)); a quiet system is a feature, not a failure.

## Flow 1 — Photo (`02-photo-flow.html`)

1. **Open:** a photo takes the Stage.
2. **Infer:** facets `image(0.98)`; the Current holds the "beach trip" thread; Memory
   says *edit → share* is this user's pattern. Local router: no cloud needed.
3. **Surface:** halo materializes — Enhance · Crop · Annotate · Remove background · Share.
4. **Predict:** Rail shows the six other photos from the same shoot (embedding + time
   cluster); next actions: *"Make a collage from these 6?" (0.74)* and *"Send to Mom"*
   (frequent recipient for this thread). Multi-select flips the halo to set-level tools.

## Flow 2 — PDF / Document (`03-pdf-flow.html`)

1. **Open:** an invoice PDF takes the Stage.
2. **Infer:** facets `document(0.99) · invoice(0.87)`. Subtype matters: invoices summon
   different tools than papers.
3. **Surface:** Read · Search-in-doc · Summarize · Extract fields · Annotate.
4. **Predict:** Rail: the sender's previous invoices; *"Extract line items to a table?"*.
   Summarize/extract route to cloud — the Surface shows the router's reason and the two
   redacted spans *before* anything leaves the device.

## Flow 3 — Code (`04-code-flow.html`)

1. **Open:** a source file takes the Stage, syntax-lit.
2. **Infer:** facets `code(0.99) · typescript(0.97)`; the Context Graph knows its imports
   and callers from Substrate indexing.
3. **Surface:** Explain · Edit · Run · Diff · Find callers.
4. **Predict:** Rail: the files this one imports and the test that covers it; next
   actions: *"Explain this function"*, *"Write a test for `parseManifest`?"*. Explain
   routes to cloud with the file excerpt only.

## Flow 4 — Text / Notes (`05-text-notes-flow.html`)

1. **Open:** a note with checkboxes takes the Stage.
2. **Infer:** facets `notes(0.95) · todo-list(0.81)` — semantic, not extension-based.
3. **Surface:** Edit · Rewrite · Summarize · Extract tasks · Link to project.
4. **Predict:** Rail: two related notes from the same project entity; next actions:
   *"Turn 3 checkboxes into reminders?"*, *"Draft this into an email?"*. Mostly local;
   rewrite escalates by dial policy.

## Flow 5 — Delegation (`08-delegation.html`)

1. **Ask:** Intent Bar: *"make a collage of the best beach photos and send it to Mom."*
2. **Plan, visibly:** the Stage becomes the task: four steps — Select · Collage ·
   Compose · Send — each naming the Capability it will use.
3. **Work, with the same hands:** steps run the same Surfaces the user would have used,
   visible, journaled, interruptible; the user can seize the controls mid-flight.
4. **Stop at the one-way door:** *Send* is `gated-irreversible`. Everything is staged —
   the collage, the message, the recipient — and the system waits. Confirm sends;
   everything before it remains undoable from the Journal.

## The Workbench view (`09-workbench.html`)

Summoned from the presence indicator (never self-opening): the AI's desk — hypotheses
with evidence and confidence awaiting confirm/correct/delete, queued Curiosity questions,
last night's Dreamtime log, and drafts of delegated work. The one place the system talks
about *itself*.

---
*Next: [08-data-knowledge-model.md](08-data-knowledge-model.md) — what everything is made of.*
