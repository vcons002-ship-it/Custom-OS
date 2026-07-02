#!/bin/sh
# Clade post-build: weaved is init. No inittab, no getty, no login.
set -eu
TARGET_DIR="$1"
ln -sf /usr/bin/weaved "$TARGET_DIR/sbin/init"
# The recovery console (docs/03-os-layer.md) is busybox sh on VT2, reachable
# by keychord only — wired up at M1; the binary ships now.
