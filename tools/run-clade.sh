#!/usr/bin/env bash
# Start Clade: boot the OS image if it is built, otherwise start the dev
# harness with a clear diagnosis of what is missing.
#
# Called by run.bat (Windows/WSL) or directly on Linux:
#     tools/run-clade.sh [ dev | headless | sdl | vnc ]
#
# Display choices (the GUI window uses GTK/XWayland by default; if it stays
# stuck in the taskbar, try a fallback with no rebuild):
#     run.bat            GTK window (default)
#     run.bat sdl        SDL window (alternate backend, also via XWayland)
#     run.bat vnc        VNC server on localhost:5900 (needs a Windows viewer)
#     run.bat headless   no window; serial only
#     run.bat dev        the dev harness (no VM)
set -euo pipefail

REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BUILDROOT_DIR="${BUILDROOT_DIR:-$REPO_DIR/../buildroot}"
IMG="$BUILDROOT_DIR/output/images"
CHOICE="${1:-}"

# Translate the friendly choice into (qemu MODE, CLADE_DISPLAY override).
QMODE=display
case "$CHOICE" in
    dev)      exec "$REPO_DIR/tools/dev-run.sh" ;;
    headless) QMODE=headless ;;
    sdl)      export CLADE_DISPLAY=sdl ;;
    vnc)      export CLADE_DISPLAY="vnc=127.0.0.1:0" ;;
    ""|gtk|display) : ;;  # GTK default
    *)        echo "  [!] unknown mode '$CHOICE' — using the default GTK window" ;;
esac

if [ -f "$IMG/bzImage" ] && [ -f "$IMG/rootfs.ext4" ]; then
    exec "$REPO_DIR/tools/qemu-run.sh" "$IMG" "$QMODE"
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
