# Building the Clade OS image

The image builds with [Buildroot](https://buildroot.org) (2025.02 LTS+ for the
`cargo-package` infrastructure) on the owner's machine or CI — not in the dev
loop. The dev loop is `tools/dev-run.sh`; see docs/phase-1-plan.md.

`tools/setup.sh` (or `setup.bat` on Windows 11) installs every prerequisite
and clones Buildroot to `../buildroot`, so normally you only need the two
`make` commands below.

```sh
git clone https://gitlab.com/buildroot.org/buildroot.git --branch 2025.02.x
cd buildroot
make BR2_EXTERNAL=/path/to/Custom-OS/kernel/buildroot-external clade_x86_64_defconfig
make    # first build downloads + compiles the toolchain and kernel: ~30-60 min
```

Artifacts land in `output/images/` (`bzImage`, `rootfs.ext4`). Boot them:

```sh
/path/to/Custom-OS/tools/qemu-run.sh output/images
```

M0's gate: the VM boots with no login and no desktop, prints the Clade
banner, brings up the five services, and logs `weave-ready`.
