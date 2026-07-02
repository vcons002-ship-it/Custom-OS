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

# GUI needs a display server (WSLg exports DISPLAY/WAYLAND_DISPLAY); without
# one, '-display gtk' hard-exits — fall back to headless with a message.
if [ "$MODE" != "headless" ] && [ -z "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]; then
    echo "[qemu-run] no display server (WSLg?) — falling back to headless/serial"
    MODE=headless
fi

# GUI mode: virtio-gpu is the ONLY display adapter (-vga none: QEMU's default
# std-VGA would otherwise be the head the window shows — blank). No gl=on:
# GL under WSLg is unreliable and we don't need 3D yet. The LAST console=
# becomes /dev/console, i.e. where weaved's banner goes: the GUI window when
# there is one, serial otherwise.
if [ "$MODE" = "headless" ]; then
    DISPLAY_ARGS="-display none"
    CONSOLES="console=tty0 console=ttyS0"
else
    DISPLAY_ARGS="-device virtio-gpu-pci -display gtk"
    CONSOLES="console=ttyS0 console=tty0"
fi

# KVM needs /dev/kvm to be openable, not merely present (kvm group).
if [ -w /dev/kvm ]; then
    KVM_ARGS="-enable-kvm -cpu host"
elif [ -e /dev/kvm ]; then
    echo "[qemu-run] /dev/kvm exists but is not writable — run tools/setup.sh"
    echo "[qemu-run] (adds you to the kvm group), then 'wsl --shutdown' and retry."
    echo "[qemu-run] Continuing unaccelerated (slow)."
    KVM_ARGS="-cpu qemu64"
else
    echo "[qemu-run] /dev/kvm missing — running unaccelerated (see tools/setup.sh KVM notes)"
    KVM_ARGS="-cpu qemu64"
fi

# Size the VM to what this environment actually has (the reference 12G only
# when the WSL VM is big enough — .wslconfig may not have applied).
VM_MEM=$(awk '/MemTotal/ { kb=$2; if (kb > 15000000) print "12G"; else if (kb > 9000000) print "6G"; else print "3G" }' /proc/meminfo)
echo "[qemu-run] VM memory: $VM_MEM"

exec qemu-system-x86_64 \
  $KVM_ARGS \
  -smp 8 \
  -m "$VM_MEM" \
  -vga none \
  -kernel "$IMAGES/bzImage" \
  -drive file="$IMAGES/rootfs.ext4",format=raw,if=virtio \
  -drive file="$DATA_DISK",format=raw,if=virtio \
  -append "root=/dev/vda rw quiet init=/sbin/init $CONSOLES" \
  -serial mon:stdio \
  -netdev user,id=n0 -device virtio-net-pci,netdev=n0 \
  $DISPLAY_ARGS
