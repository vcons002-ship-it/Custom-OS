################################################################################
# clade — installs the prebuilt, static (musl) Clade binaries as the userland
#
# The binaries are built OUTSIDE Buildroot by the build script
# (build-image.bat / kernel/README.md): `cargo build --release --target
# x86_64-unknown-linux-musl --locked` on the host Rust toolchain. Static
# binaries run on any Linux, so the image toolchain and the Rust toolchain
# are fully decoupled: Buildroot only assembles kernel + skeleton + these
# files. (The previous cargo-package/local-site approach both rsynced the
# multi-GB target/ dir into the build and fought Buildroot's offline
# vendoring; this is simpler and faster.)
################################################################################

CLADE_VERSION = 0.1.0
CLADE_SOURCE =
CLADE_LICENSE = MIT

CLADE_REPO = $(BR2_EXTERNAL_CLADE_PATH)/../..
CLADE_BINDIR = $(CLADE_REPO)/target/x86_64-unknown-linux-musl/release
CLADE_BINARIES = weaved weave cortexd substrated modeld gated capd

define CLADE_INSTALL_TARGET_CMDS
	for bin in $(CLADE_BINARIES); do \
		if [ ! -x $(CLADE_BINDIR)/$$bin ]; then \
			echo "clade: missing $(CLADE_BINDIR)/$$bin"; \
			echo "clade: run the cargo musl build first (build-image.bat does this automatically)"; \
			exit 1; \
		fi; \
		$(INSTALL) -D -m 0755 $(CLADE_BINDIR)/$$bin $(TARGET_DIR)/usr/bin/$$bin || exit 1; \
	done
	mkdir -p $(TARGET_DIR)/run/clade $(TARGET_DIR)/data
endef

$(eval $(generic-package))
