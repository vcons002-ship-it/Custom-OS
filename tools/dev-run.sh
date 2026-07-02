#!/bin/sh
# The dev harness: run weaved + the mind plane on a normal Linux host —
# no VM, no image build. Same binaries, same bus, no PID-1 duties.
set -eu
cd "$(dirname "$0")/.."

export CLADE_BUS="${CLADE_BUS:-/tmp/clade-dev/bus.sock}"
mkdir -p "$(dirname "$CLADE_BUS")"

cargo build --workspace
echo "[dev-run] bus at $CLADE_BUS — Ctrl-C stops everything"
exec ./target/debug/weaved
