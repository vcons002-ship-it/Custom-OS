# Building the Clade OS image

Two steps, in this order (build-image.bat / CI do both automatically):

1. **Clade's binaries** — built on the host Rust toolchain as **static musl**
   executables, so they run in the image regardless of its libc:

   ```sh
   rustup target add x86_64-unknown-linux-musl   # once; setup.sh does this
   cargo build --release --target x86_64-unknown-linux-musl --locked
   ```

2. **The image** — [Buildroot](https://buildroot.org) (2025.02 LTS+) assembles
   the kernel + minimal rootfs and installs the binaries from step 1. It uses a
   prebuilt Bootlin external toolchain, so the first build is dominated by the
   kernel compile (~15–40 min depending on cores), then cached:

   ```sh
   git clone https://gitlab.com/buildroot.org/buildroot.git --branch 2025.02.x
   cd buildroot
   make BR2_EXTERNAL=/path/to/Custom-OS/kernel/buildroot-external clade_x86_64_defconfig
   make
   ```

`tools/setup.sh` (or `setup.bat` on Windows 11) installs every prerequisite and
clones Buildroot to `../buildroot`. **Buildroot refuses to run if PATH contains
spaces** — on WSL the Windows PATH is appended by default, so export a clean
PATH first (build-image.bat does): 

```sh
export PATH=$HOME/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
```

Artifacts land in `output/images/` (`bzImage`, `rootfs.ext4`). Boot them:

```sh
/path/to/Custom-OS/tools/qemu-run.sh output/images            # GUI window
/path/to/Custom-OS/tools/qemu-run.sh output/images headless   # serial only
```

M0's gate: the VM boots with no login and no desktop, prints the Clade
banner, brings up the five services, and logs `weave-ready`.
