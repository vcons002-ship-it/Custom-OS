#!/usr/bin/env bash
# Clade — build the bootable OS image.
#
# Called by build-image.bat (Windows/WSL) or directly on Linux:
#     tools/build-image.sh
#
# Two steps (docs in kernel/README.md):
#   1. Clade's binaries: static musl, built by the host Rust toolchain.
#   2. Buildroot assembles kernel + rootfs and installs those binaries.
#
# Idempotent and resumable: Buildroot caches everything it has already built.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BUILDROOT_DIR="${BUILDROOT_DIR:-$REPO_DIR/../buildroot}"

step() { printf '\n\033[1;36m[build-image]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[build-image]\033[0m %s\n' "$*"; }
die()  { printf '\033[1;31m[build-image]\033[0m %s\n' "$*" >&2; exit 1; }

# Buildroot refuses a PATH containing spaces — WSL appends the Windows PATH
# by default, so build with a clean, known-good one.
export PATH="$HOME/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

# Advisory disk-space check: a first image build wants ~15GB headroom.
AVAIL_KB=$(df -Pk "$HOME" | awk 'NR==2 {print $4}')
if [ "${AVAIL_KB:-0}" -lt 15000000 ]; then
    warn "less than 15GB free in the WSL disk — the first image build may run out of space"
fi

command -v cargo >/dev/null 2>&1 || die "cargo not found — run tools/setup.sh (or install-deps.bat) first"
[ -d "$BUILDROOT_DIR" ] || die "Buildroot not found at $BUILDROOT_DIR — run tools/setup.sh (or install-deps.bat) first"

step "building Clade's static binaries (musl)"
rustup target add x86_64-unknown-linux-musl >/dev/null 2>&1 || true
cd "$REPO_DIR"
cargo build --release --target x86_64-unknown-linux-musl --locked

step "assembling the OS image (Buildroot)"
cd "$BUILDROOT_DIR"
make BR2_EXTERNAL="$REPO_DIR/kernel/buildroot-external" clade_x86_64_defconfig
# Buildroot's install stamps don't notice rebuilt binaries — force reinstall
# so the image always carries what step 1 just built.
make clade-reinstall
make

step "image ready: $BUILDROOT_DIR/output/images (bzImage, rootfs.ext4)"
step "boot it with run.bat (Windows) or tools/qemu-run.sh (Linux)"
