# Main building script

include scripts/make/cargo.mk

ifeq ($(APP_TYPE), c)
  include scripts/make/build_c.mk
else ifeq ($(APP_TYPE), rust)
  rust_package := $(shell cat $(APP)/Cargo.toml | sed -n 's/name = "\([a-z0-9A-Z_\-]*\)"/\1/p')
  rust_target_dir := $(CURDIR)/target/$(TARGET)/$(MODE)
  rust_elf := $(rust_target_dir)/$(rust_package)
else
  $(error "APP_TYPE" must be one of "rust" or "c")
endif

_cargo_build:
	@printf "    $(GREEN_C)Building$(END_C) App: $(APP_NAME), Arch: $(ARCH), Platform: $(PLATFORM), App type: $(APP_TYPE)\n"

ifeq ($(APP_TYPE), rust)
	$(if $(V), $(info RUSTFLAGS: "$(RUSTFLAGS)"))
	$(call cargo_rustc,--manifest-path $(APP)/Cargo.toml)
	@cp $(rust_elf) $(OUT_ELF)
else ifeq ($(APP_TYPE), c)
	$(if $(V), $(info CFLAGS: "$(CFLAGS)") $(info LDFLAGS: "$(LDFLAGS)"))
	$(call cargo_rustc,-p libax)
endif

$(OUT_DIR):
	$(call run_cmd,mkdir,-p $@)

$(OUT_BIN): _cargo_build $(OUT_ELF)
	$(call run_cmd,$(OBJCOPY),$(OUT_ELF) --strip-all -O binary $@)

.PHONY: _cargo_build
