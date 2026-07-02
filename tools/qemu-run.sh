#!/bin/sh
# Boot the Clade image in QEMU/KVM — the Phase-1 reference machine
# (docs/phase-1-plan.md): 8 vCPUs, 12G RAM, virtio-gpu, KVM-accelerated.
#
# Usage: tools/qemu-run.sh <buildroot output/images dir> [headless]
set -eu

IMAGES="${1:?usage: qemu-run.sh <images-dir> [headless]}"
MODE="${2:-display}"

DISPLAY_ARGS="-device virtio-gpu-pci -display gtk,gl=on"
[ "$MODE" = "headless" ] && DISPLAY_ARGS="-display none"

exec qemu-system-x86_64 \
  -enable-kvm \
  -cpu host \
  -smp 8 \
  -m 12G \
  -kernel "$IMAGES/bzImage" \
  -drive file="$IMAGES/rootfs.ext4",format=raw,if=virtio \
  -append "root=/dev/vda rw quiet init=/sbin/init console=ttyS0" \
  -serial mon:stdio \
  -netdev user,id=n0 -device virtio-net-pci,netdev=n0 \
  $DISPLAY_ARGS
