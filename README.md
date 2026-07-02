# Custom-OS — "Clade": a living, AI-native operating system

> **The AI is the computer.** You never open a program. You open content, and the tools
> you need materialize around it — shaped by what the content is and what you tend to do
> with it. When you move on, they dissolve. The system's mind is resident: it holds
> context across your day, consolidates while the machine sleeps, asks when it's unsure,
> and can be handed whole tasks. It is its own OS — booted directly, with no desktop,
> no windows, no apps underneath.

This repository is the **concept and architecture foundation** for that operating system:
a complete design-phase package — rigorous documentation plus high-fidelity clickable
mockups — that the build phase executes against. There is intentionally no system code yet.

## Read the design

Start at the vision and read in order; each document builds on the vocabulary of the last.

| Doc | Contents |
|---|---|
| [00-vision.md](docs/00-vision.md) | The living computer: thesis, principles, non-goals |
| [01-glossary.md](docs/01-glossary.md) | The lexicon everything else is written in + product-name shortlist |
| [02-system-architecture.md](docs/02-system-architecture.md) | The full stack, kernel → Weave → Surfaces, and the canonical data flow |
| [03-os-layer.md](docs/03-os-layer.md) | Its own OS: boot/init, the Weave as PID 1, hardware abstraction, userland replacement |
| [04-ai-architecture.md](docs/04-ai-architecture.md) | The eight AI subsystems and how they interact |
| [05-resident-mind.md](docs/05-resident-mind.md) | The Current, Dreamtime, the Workbench, Curiosity, Delegation, the attention economy |
| [06-hybrid-ai.md](docs/06-hybrid-ai.md) | On-device vs. cloud, the router, the Capability manifest, Materialize/Dissolve |
| [07-interaction-model.md](docs/07-interaction-model.md) | The Weave's zones and the end-to-end flows (photo, PDF, code, notes, delegation) |
| [08-data-knowledge-model.md](docs/08-data-knowledge-model.md) | The Substrate, Context Graph, Memory layers, and the Journal |
| [09-privacy-trust.md](docs/09-privacy-trust.md) | An OS you own: Privacy Dial, redaction gate, consent, universal undo |
| [10-tech-stack.md](docs/10-tech-stack.md) | Build-phase stack: OS image, Rust services, compositor, model runtimes |
| [11-roadmap.md](docs/11-roadmap.md) | Concept → bootable seed → resident mind → ecosystem, with phase gates |
| [12-risks-open-questions.md](docs/12-risks-open-questions.md) | Hard problems, stated honestly, and open decisions |

## Feel the design

The mockups are self-contained HTML/CSS/JS — no build step, no network, no dependencies.

```
open mockups/index.html        # macOS
xdg-open mockups/index.html    # Linux
```

Click through the screens in order: boot → the idle Weave → the four content flows →
the Materialize/Dissolve demo → the Privacy Dial → Delegation → the Workbench. Each
screen demonstrates one path through the morphing UI described in
[07-interaction-model.md](docs/07-interaction-model.md).

## Status

**Phase 0 — Concept (this repository).** See [11-roadmap.md](docs/11-roadmap.md) for what
Phase 1 (a bootable seed in QEMU) requires and the gates between phases.

## License

MIT — see [LICENSE](LICENSE).
