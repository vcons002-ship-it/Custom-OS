#!/bin/sh
# Boot the Clade image in QEMU/KVM — the Phase-1 reference machine
# (docs/phase-1-plan.md): 8 vCPUs, 12G RAM, virtio-gpu, KVM-accelerated.
#
# Two disks:
#   /dev/vda — the OS image (rootfs.ext4). Rebuilt/replaced by Buildroot.
#   /dev/vdb — the PERSISTENT DATA VOLUME (auto-created here, 8G ext4).
#              Mounted at /data inside Clade; holds everything that must
#              survive sessions and OS-image rebuilds: the Substrate index,
#              Context Graph, Memory, the Current, and the Journal.
#
# Usage: tools/qemu-run.sh <buildroot output/images dir> [headless]
set -eu

IMAGES="${1:?usage: qemu-run.sh <images-dir> [headless]}"
MODE="${2:-display}"

# Persistent data volume — a plain file on the host; delete it to factory-reset
# Clade's mind without touching the OS image (or your host).
DATA_DISK="${CLADE_DATA:-$HOME/clade/clade-data.img}"
if [ ! -f "$DATA_DISK" ]; then
    echo "[qemu-run] creating persistent data volume: $DATA_DISK (8G, ext4)"
    mkdir -p "$(dirname "$DATA_DISK")"
    truncate -s 8G "$DATA_DISK"
    mkfs.ext4 -q -F "$DATA_DISK"
fi

DISPLAY_ARGS="-device virtio-gpu-pci -display gtk,gl=on"
[ "$MODE" = "headless" ] && DISPLAY_ARGS="-display none"

KVM_ARGS="-enable-kvm -cpu host"
[ -e /dev/kvm ] || {
    echo "[qemu-run] /dev/kvm missing — running unaccelerated (see tools/setup.sh KVM notes)"
    KVM_ARGS="-cpu qemu64"
}

exec qemu-system-x86_64 \
  $KVM_ARGS \
  -smp 8 \
  -m 12G \
  -kernel "$IMAGES/bzImage" \
  -drive file="$IMAGES/rootfs.ext4",format=raw,if=virtio \
  -drive file="$DATA_DISK",format=raw,if=virtio \
  -append "root=/dev/vda rw quiet init=/sbin/init console=ttyS0" \
  -serial mon:stdio \
  -netdev user,id=n0 -device virtio-net-pci,netdev=n0 \
  $DISPLAY_ARGS
