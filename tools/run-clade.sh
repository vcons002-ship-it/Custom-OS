#!/usr/bin/env bash
# Start Clade: boot the OS image if it is built, otherwise start the dev
# harness with a clear diagnosis of what is missing.
#
# Called by run.bat (Windows/WSL) or directly on Linux:
#     tools/run-clade.sh [dev|headless]
set -euo pipefail

REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BUILDROOT_DIR="${BUILDROOT_DIR:-$REPO_DIR/../buildroot}"
IMG="$BUILDROOT_DIR/output/images"
MODE="${1:-}"

if [ "$MODE" = "dev" ]; then
    exec "$REPO_DIR/tools/dev-run.sh"
fi

if [ -f "$IMG/bzImage" ] && [ -f "$IMG/rootfs.ext4" ]; then
    # shellcheck disable=SC2086 — empty MODE must vanish, not become ""
    exec "$REPO_DIR/tools/qemu-run.sh" "$IMG" $MODE
fi

echo
echo "  [*] The Clade OS image is not built (or incomplete) - contents of"
echo "      the images directory ($IMG):"
ls -la "$IMG" 2>/dev/null || echo "      (no images directory yet)"
echo
echo "  [*] Run build-image.bat (Windows) or tools/build-image.sh (Linux) to"
echo "      build or finish it - resumable and cached - then start Clade again."
echo "  [*] Starting the dev harness in the meantime..."
echo
exec "$REPO_DIR/tools/dev-run.sh"
