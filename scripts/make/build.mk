# Main building script

$(OUT_DIR):
	mkdir -p $@

ifeq ($(APP_LANG), c)
  include ulib/c_libax/build.mk
else
  rust_package := $(shell cat $(APP)/Cargo.toml | sed -n 's/name = "\([a-z0-9A-Z_\-]*\)"/\1/p')
  rust_target_dir := $(CURDIR)/target/$(TARGET)/$(MODE)
  rust_elf := $(rust_target_dir)/$(rust_package)
endif

_cargo_build:
	@echo -e "    $(GREEN_C)Building$(END_C) App: $(APP_NAME), Arch: $(ARCH), Platform: $(PLATFORM), Language: $(APP_LANG)"
ifeq ($(APP_LANG), rust)
	$(call cargo_build,--manifest-path $(APP)/Cargo.toml)
	@cp $(rust_elf) $(OUT_ELF)
else ifeq ($(APP_LANG), c)
	$(call cargo_build,-p libax)
endif

$(OUT_BIN): _cargo_build $(OUT_ELF)
	$(OBJCOPY) $(OUT_ELF) --strip-all -O binary $@

.PHONY: _cargo_build
