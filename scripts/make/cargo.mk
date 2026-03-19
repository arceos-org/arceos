# Cargo features and build args

ifeq ($(V),1)
  verbose := -v
else ifeq ($(V),2)
  verbose := -vv
else
  verbose :=
endif

build_args-release := --release

build_args := \
  -Z unstable-options \
  --target $(TARGET) \
  --target-dir $(TARGET_DIR) \
  $(build_args-$(MODE)) \
  $(verbose)

RUSTFLAGS := -A unsafe_op_in_unsafe_fn

ifeq ($(PLAT_DYN),y)
  build_args += -Z build-std=core,alloc
endif

ifeq ($(PLAT_DYN),y)
  RUSTFLAGS_LINK_ARGS := -C relocation-model=pic -C link-arg=-pie -C link-arg=-znostart-stop-gc -C link-arg=-Taxplat.x
else
  RUSTFLAGS_LINK_ARGS := -C link-arg=-Tlinker.x -C link-arg=-no-pie -C link-arg=-znostart-stop-gc
endif
RUSTDOCFLAGS := -Z unstable-options --enable-index-page -D rustdoc::broken_intra_doc_links

ifeq ($(MAKECMDGOALS), doc_check_missing)
  RUSTDOCFLAGS += -D missing-docs
endif

define cargo_build
  $(call run_cmd,cargo -C $(1) build,$(build_args) --features "$(strip $(2))")
endef

clippy_args := -A unsafe_op_in_unsafe_fn

define cargo_clippy
  $(call run_cmd,cargo clippy,--workspace --exclude axlog --exclude axplat-dyn --exclude "arceos-*" $(1) $(verbose) -- $(clippy_args))
  $(call run_cmd,cargo clippy,-p axlog $(1) $(verbose) -- $(clippy_args))
endef

all_packages := \
  $(filter-out axplat-dyn,$(shell ls $(CURDIR)/modules)) \
  axfeat arceos_api axstd axlibc

define cargo_doc
  $(call run_cmd,cargo doc,--no-deps --all-features --workspace --exclude "arceos-*" --exclude axplat-dyn $(verbose))
  @# run twice to fix broken hyperlinks
  $(foreach p,$(all_packages), \
    $(call run_cmd,cargo rustdoc,--all-features -p $(p) $(verbose))
  )
endef

define unit_test
  $(call run_cmd,cargo test,-p axfs $(1) $(verbose) -- --nocapture)
  $(call run_cmd,cargo test,--workspace --exclude axfs --exclude axplat-dyn $(1) $(verbose) -- --nocapture)
endef
