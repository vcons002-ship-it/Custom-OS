################################################################################
# clade — the Rust workspace, installed as the entire userland
################################################################################

CLADE_VERSION = 0.1.0
CLADE_SITE = $(BR2_EXTERNAL_CLADE_PATH)/../..
CLADE_SITE_METHOD = local
CLADE_LICENSE = MIT
CLADE_LICENSE_FILES = LICENSE
CLADE_BINARIES = weaved weave cortexd substrated modeld gated capd

define CLADE_BUILD_CMDS
	cd $(@D) && $(PKG_CARGO_ENV) cargo build --release --locked
endef

define CLADE_INSTALL_TARGET_CMDS
	$(foreach bin,$(CLADE_BINARIES), \
		$(INSTALL) -D -m 0755 $(@D)/target/release/$(bin) $(TARGET_DIR)/usr/bin/$(bin);)
	mkdir -p $(TARGET_DIR)/run/clade $(TARGET_DIR)/data
endef

$(eval $(cargo-package))
