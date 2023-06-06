# Main building script
#
# Modification for microkernel version

$(OUT_DIR):
	mkdir -p $@

ifeq ($(APP_LANG), c)
  include ulib/c_libax/build.mk
else
  ifeq ($(MICRO_TEST), )
    rust_package := $(shell cat $(APP)/Cargo.toml | sed -n 's/name = "\([a-z0-9A-Z_\-]*\)"/\1/p')
  else
    rust_package := $(MICRO_TEST)
  endif
  rust_target_dir := $(CURDIR)/target/$(TARGET)/$(MODE)
  rust_elf := $(rust_target_dir)/$(rust_package)
endif


_cargo_build_user:
	@printf "    $(GREEN_C)Building$(END_C) App: $(APP_NAME), Arch: $(ARCH), Platform: $(PLATFORM), Language: $(APP_LANG)\n"
ifeq ($(APP_LANG), rust)
	$(call cargo_build_user,--manifest-path $(APP)/Cargo.toml --bin $(rust_package))
	@cp $(rust_elf) $(CURDIR)/modules/axuser/user.elf
else ifeq ($(APP_LANG), c)
#$(call cargo_build,-p libax)
	$(error microkernel for C apps are not supported!)
endif


_cargo_build_kern: _cargo_build_user $(CURDIR)/modules/axuser/user.elf
	@printf "    $(GREEN_C)Building$(END_C) Kernel: Arch: $(ARCH), Platform: $(PLATFORM), Language: $(APP_LANG)\n"
	$(call cargo_build_kern,--manifest-path $(CURDIR)/modules/axuser/Cargo.toml)
	@cp $(rust_target_dir)/axuser $(OUT_ELF)

$(OUT_BIN): _cargo_build_kern $(OUT_ELF)
	$(OBJCOPY) $(OUT_ELF) --strip-all -O binary $@

.PHONY: _cargo_build_kern _cargo_build_user
