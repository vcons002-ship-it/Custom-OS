#!/usr/bin/env bash
# Clade — Linux/WSL2 setup: install every dependency and verify the dev loop.
#
# Works on Ubuntu/Debian (native or WSL2). Idempotent — re-run freely; each
# step skips itself when already satisfied. Scope (docs/phase-1-plan.md):
# dependencies + the fast dev-loop verification. The ~30-60 min Buildroot
# image build is printed as the next command, never started here.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BUILDROOT_DIR="${BUILDROOT_DIR:-$REPO_DIR/../buildroot}"
BUILDROOT_BRANCH="2025.02.x"

step()  { printf '\n\033[1;36m[clade-setup]\033[0m %s\n' "$*"; }
warn()  { printf '\033[1;33m[clade-setup]\033[0m %s\n' "$*"; }
die()   { printf '\033[1;31m[clade-setup]\033[0m %s\n' "$*" >&2; exit 1; }

SUDO=""
if [ "$(id -u)" -ne 0 ]; then
    command -v sudo >/dev/null || die "need root or sudo to install packages"
    SUDO="sudo"
fi

# ---- 1. Distro packages ----------------------------------------------------
# Buildroot prerequisites + QEMU for the reference machine.
step "installing distro packages (apt)"
command -v apt-get >/dev/null || die "this script supports apt-based distros (Ubuntu/Debian/WSL2 Ubuntu)"
export DEBIAN_FRONTEND=noninteractive
$SUDO apt-get update -qq
$SUDO apt-get install -y -qq \
    build-essential git curl file wget cpio unzip rsync bc \
    libncurses-dev libssl-dev pkg-config \
    qemu-system-x86 ovmf \
    >/dev/null
step "packages ok"

# ---- 2. KVM probe (warn only) ----------------------------------------------
if [ -e /dev/kvm ]; then
    step "KVM available — QEMU will run hardware-accelerated"
else
    warn "/dev/kvm not found — QEMU will run unaccelerated (slow but works)."
    warn "On Windows 11/WSL2: ensure %USERPROFILE%\\.wslconfig has"
    warn "  [wsl2]"
    warn "  nestedVirtualization=true"
    warn "then 'wsl --shutdown' from Windows and reopen. Also check the BIOS:"
    warn "AMD 'SVM Mode' must be enabled. (setup.bat does the .wslconfig part.)"
fi

# ---- 3. Rust ----------------------------------------------------------------
if command -v cargo >/dev/null 2>&1; then
    step "Rust present: $(cargo --version)"
else
    step "installing Rust (rustup)"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
    step "Rust installed: $(cargo --version)"
fi
# Make cargo visible for the rest of this script even on fresh installs.
export PATH="$HOME/.cargo/bin:$PATH"
rustup component add rustfmt clippy >/dev/null 2>&1 || true

# ---- 4. Buildroot (fetch now, build later) ----------------------------------
if [ -d "$BUILDROOT_DIR/.git" ]; then
    step "Buildroot already at $BUILDROOT_DIR"
else
    step "fetching Buildroot ($BUILDROOT_BRANCH) to $BUILDROOT_DIR (shallow)"
    git clone --depth 1 --branch "$BUILDROOT_BRANCH" \
        https://gitlab.com/buildroot.org/buildroot.git "$BUILDROOT_DIR"
fi

# ---- 5. Verify the dev loop --------------------------------------------------
step "building the Clade workspace"
cd "$REPO_DIR"
cargo build --workspace

step "running the test suite"
cargo test --workspace --quiet

step "booting the dev harness (banner → services → weave-ready)"
export CLADE_BUS="${CLADE_BUS:-/tmp/clade-setup/bus.sock}"
mkdir -p "$(dirname "$CLADE_BUS")"
rm -f "$CLADE_BUS"
HARNESS_LOG="$(mktemp)"
timeout 10 ./target/debug/weaved >"$HARNESS_LOG" 2>&1 || true  # timeout kill is the expected exit
if grep -q "weave-ready" "$HARNESS_LOG"; then
    step "dev loop verified — weaved reached weave-ready:"
    grep -E "C L A D E|weave-ready|up \(pid" "$HARNESS_LOG" | sed 's/^/    /'
else
    warn "harness log follows:"
    cat "$HARNESS_LOG"
    die "the dev harness did not reach weave-ready"
fi
rm -f "$HARNESS_LOG"

# ---- 6. Done: next steps ------------------------------------------------------
cat <<EOF

  ============================================================
   Clade is set up. Everything for Phase 1 is installed.

   Fast dev loop (runs the mind plane on this host):
       tools/dev-run.sh

   Build the OS image (first build ~30-60 min, then cached):
       cd $BUILDROOT_DIR
       make BR2_EXTERNAL=$REPO_DIR/kernel/buildroot-external clade_x86_64_defconfig
       make

   Boot your own OS:
       $REPO_DIR/tools/qemu-run.sh $BUILDROOT_DIR/output/images

   Later (M6): gated reaches your host Ollama server from inside
   WSL2/QEMU via the host address — no key needed, your GPU is
   the escalation tier. Gemini API is the opt-in fallback.
  ============================================================
EOF
