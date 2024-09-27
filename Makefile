#
# Copyright 2024, DornerWorks
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

# BUILD directory needs to match `build.artifact-dir` in .cargo/config.toml
BUILD ?= build
MICROKIT_BOARD ?= zcu102

system_description := zcu102_server.system
build_dir := $(BUILD)
loader := $(build_dir)/loader.img

microkit_board := $(MICROKIT_BOARD)
microkit_config := debug
microkit_sdk_config_dir := $(MICROKIT_SDK)/board/$(microkit_board)/$(microkit_config)

sel4_include_dirs := $(microkit_sdk_config_dir)/include

.PHONY: loader
loader: $(loader)

### Protection domains

crate = $(build_dir)/$(1).elf

define build_crate

$(crate): $(crate).intermediate

.INTERMDIATE: $(crate).intermediate
$(crate).intermediate:
	SEL4_INCLUDE_DIRS=$(abspath $(sel4_include_dirs)) \
		cargo build \
			-Z build-std=core,alloc,compiler_builtins \
			-Z build-std-features=compiler-builtins-mem \
			--release \
			-p $(1)

endef

crate_names := \
	ping \
	eth-driver

crates := $(foreach crate_name,$(crate_names),$(call crate,$(crate_name)))

$(eval $(foreach crate_name,$(crate_names),$(call build_crate,$(crate_name))))

.PHONY: build_crates
build_crates: $(crates)

### Loader
$(loader): $(system_description) build_crates
	$(MICROKIT_SDK)/bin/microkit \
		$< \
		--search-path $(build_dir) \
		--board $(microkit_board) \
		--config $(microkit_config) \
		-r $(build_dir)/report.txt \
		-o $@


.PHONY: clean
clean:
	rm -rf $(build_dir)
