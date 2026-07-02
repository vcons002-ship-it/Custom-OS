# 00 — Vision: The Living Computer

> **Working title: Morph** (provisional — see the shortlist in [01-glossary.md](01-glossary.md)).

## The problem with computers

Every operating system you have ever used is organized around **programs**. Content is
secondary — a photo is "a file you open *with* something," a PDF is "a document that
*belongs to* a reader app." To do anything, you must first answer a question the computer
should be answering for you: *which tool do I need?* Then you launch it, wait, arrange its
window, find your content inside it, and only then begin the thing you actually wanted to do.

Apps are silos. Windows are scaffolding. File pickers, launchers, docks, and taskbars are
all workarounds for one design flaw: **the computer does not understand what you are doing.**

## The thesis

**The AI is the computer.**

Morph is a new kind of operating system — a *living computer* — built on three inversions:

1. **Content first, tools second.** You never open a program. You open *content* — a photo,
   an invoice, a source file, a note — and the tools you need materialize around it, shaped
   by what the content is and what you tend to do with it. When you move on, they dissolve.
   Tools have no fixed form, no icon, no window. They exist when needed and not when not.

2. **The filesystem is navigated by meaning.** Files are just handles to content. The system
   understands what each item *is* (its Facets), what it relates to, who it involves, and
   when it mattered. "The invoice from March" is a valid address. Opening one beach photo
   surfaces the rest of the shoot — because the computer understands they belong together.

3. **The mind is resident, not summoned.** This is what makes it *living*, and it is the
   difference between an OS with AI features and an AI that is the OS. The system's
   intelligence — the Cortex — runs continuously. It holds a persistent working context
   (the Current), consolidates and reflects while the machine idles (Dreamtime), keeps an
   inspectable workspace of what it believes about you (the Workbench), asks when it is
   unsure instead of guessing (Curiosity), and can be handed whole tasks to carry out with
   the same tools it offers you (Delegation). **Continuity — of context, of memory, of
   relationship — is the property that makes the computer alive.**

## It is its own OS

Morph is not an application, a launcher, or a skin over a desktop. It is a **standalone
operating system you boot into**. A bare Linux kernel is used strictly as the hardware
layer — drivers, memory, scheduling, networking — and *everything above it is replaced*.
There is no desktop environment, no window manager, no shell prompt, no app model. The
machine boots directly into the Weave, Morph's native environment. Nothing about the
architecture is constrained by another system's programs or assumptions.

See [03-os-layer.md](03-os-layer.md) for how this works.

## Design principles

| Principle | Meaning |
|---|---|
| **Content-first** | The thing you care about holds the center (the Stage). Tools orbit it. |
| **Calm** | The system earns attention; it doesn't demand it. Interruptions draw from a finite budget. |
| **Legible** | Every suggestion, every cloud call, every learned habit shows its *why* — and can be corrected. |
| **Continuous** | Nothing launches, nothing quits. The environment morphs. The mind persists. |
| **Owned** | Your content, your memory of you, your machine. Local-first; the cloud is an invited guest. |
| **Reversible** | Every action — the AI's and yours — lands in the Journal and can be undone. Boldness is safe because nothing is permanent. |

## Non-goals

- **Not a chatbot skin.** Conversation is one input among many; the primary interface is
  content and the tools that form around it.
- **Not an app platform.** There are no apps to install. There are Capabilities — declarative
  tool manifests the system materializes as needed.
- **Not a cloud terminal.** Heavy reasoning may use the cloud, by policy and with consent,
  but the OS is fully functional — degraded, honestly — with the network cable cut.
- **Not a surveillance machine.** The system's model of you exists to serve you, is stored
  on your machine, is readable and editable by you, and can be wiped by you.

## What success looks like

You sit down. The machine already indexed last night's photos, noticed they cluster with
the "beach trip" thread from your messages, and quietly pre-warmed the tools you use on
photos. You open one. The rest of the shoot appears beside it, editing tools form around
it, and a modest suggestion sits in the rail: *"Make a collage from these six?"* — with a
small *why* you can expand and a dismissal it will actually learn from. You say, "send the
best ones to Mom." It selects, composes, shows you the message, and waits at the one step
it cannot undo. That is the living computer.

---
*Next: [01-glossary.md](01-glossary.md) — the vocabulary this entire design is written in.*
