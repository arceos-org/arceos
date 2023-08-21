# Building script for stdapps

host_target := $(shell rustc -vV | grep host | cut -d: -f2 | tr -d " ")

sysroot := $(CURDIR)/sysroot
rustlib_dir := $(sysroot)/lib/rustlib/$(TARGET)
rustlib_host_dir := $(sysroot)/lib/rustlib/$(host_target)
rust_src := $(CURDIR)/third_party/rust

build_std_args := \
  --target $(TARGET) \
  --release \
  --manifest-path $(rust_src)/library/std/Cargo.toml \
  $(verbose)

RUSTFLAGS += \
  --sysroot $(sysroot) \
  -C embed-bitcode=yes \
  -Z force-unstable-if-unmarked

$(rustlib_dir):
	@printf "    $(GREEN_C)Creating$(END_C) sysroot\n"
	$(call run_cmd,mkdir,-p $(rustlib_dir) $(rustlib_host_dir))
	$(call run_cmd,ln,-sf $(rust_src)/target/$(TARGET)/release/deps $(rustlib_dir)/lib)
	$(call run_cmd,ln,-sf $(rust_src)/target/release/deps $(rustlib_host_dir)/lib)
	$(call run_cmd,ln,-sf $(CURDIR)/target_spec/$(TARGET).json $(rustlib_dir)/target.json)

build_std: $(rustlib_dir)
	@printf "    $(GREEN_C)Building$(END_C) rust-std ($(TARGET))\n"
#	stage 1: build the core and alloc libraries first which are required by ArceOS
	$(call run_cmd,cargo build,$(build_std_args) -p core -p alloc --features "compiler-builtins-mem")
#	stage 2: build ArceOS and the std library with specified features
	$(call run_cmd,cargo build,$(build_std_args) -p std --features "compiler-builtins-mem $(AX_FEAT) $(LIB_FEAT)")
