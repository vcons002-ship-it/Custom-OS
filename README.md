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

## Set up on your machine

**Windows 11** (three .bat entry points, run from this folder):

| Script | What it does |
|---|---|
| `setup.bat` | First-time setup (run as Administrator once): installs WSL2 + Ubuntu with nested virtualization (so QEMU gets KVM), copies the repo into WSL, and runs the Linux installer. A reboot may be needed once; re-run after and it resumes. |
| `install-deps.bat` | Syncs the checkout into WSL and re-runs the Linux dependency installer (`tools/setup.sh`). Run after pulls that change dependencies — it's idempotent, so when in doubt, run it. |
| `build-image.bat` | Builds the bootable Clade OS image (Buildroot). First build ~30–60 min, then cached; safe to interrupt and resume. Run once, and again whenever the OS recipe (`kernel/`) changes. |
| `run.bat` | Starts Clade: syncs the checkout into WSL, then boots the real OS image in QEMU if built, otherwise starts the dev harness and prints the image-build command. `run.bat dev` forces the harness; `run.bat headless` boots without a display window. |

Day-to-day order: **`setup.bat` once ever** → **`build-image.bat` once** →
**`run.bat` every time** (it picks up `git pull` automatically). `install-deps.bat`
only when dependencies change; `build-image.bat` again only when `kernel/` changes.

**If the QEMU window stays stuck in the taskbar and won't open** (a WSLg/GTK
quirk), try a fallback backend — no rebuild needed:

| Command | Window |
|---|---|
| `run.bat` | GTK, via XWayland (default; fixes the stuck-window case) |
| `run.bat sdl` | SDL backend (try if GTK still won't map) |
| `run.bat vnc` | VNC server on `localhost:5900` (connect a Windows VNC viewer) |
| `run.bat headless` | No window; the full boot streams in the terminal |

If none open a window, refresh WSLg once from an elevated PowerShell:
`wsl --shutdown` then `wsl --update`, and try again.

**Linux** — `tools/setup.sh` to set up, `tools/qemu-run.sh <images-dir>` to boot,
`tools/dev-run.sh` for the harness.

Setup installs the toolchain (Rust, QEMU, Buildroot prerequisites), fetches Buildroot,
builds and tests the workspace, and proves the dev loop by booting the `weaved` harness
to `weave-ready`. The full OS-image build (~30–60 min once) is documented in
[kernel/README.md](kernel/README.md).

**Persistence:** Clade's mind lives on a dedicated data volume
(`~/clade/clade-data.img` inside WSL, auto-created at first boot, mounted at `/data`
in the OS). It survives shutdowns and OS-image rebuilds; delete the file to
factory-reset the mind without touching your files or the OS image.

## Status

**Phase 0 — Concept (this repository).** See [11-roadmap.md](docs/11-roadmap.md) for what
Phase 1 (a bootable seed in QEMU) requires and the gates between phases.

## License

MIT — see [LICENSE](LICENSE).
