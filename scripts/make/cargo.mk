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

RUSTFLAGS:= -A unsafe_op_in_unsafe_fn
RUSTFLAGS_LINK_ARGS := -C link-arg=-T$(LD_SCRIPT) -C link-arg=-no-pie -C link-arg=-znostart-stop-gc
RUSTDOCFLAGS := -Z unstable-options --enable-index-page -D rustdoc::broken_intra_doc_links

ifeq ($(MAKECMDGOALS), doc_check_missing)
  RUSTDOCFLAGS += -D missing-docs
endif

define cargo_build
  $(call run_cmd,cargo -C $(1) build,$(build_args) --features "$(strip $(2))")
endef

clippy_args := -A clippy::new_without_default -A unsafe_op_in_unsafe_fn

define cargo_clippy
  $(call run_cmd,cargo clippy,--all-features --workspace --exclude axlog $(1) $(verbose) -- $(clippy_args))
  $(call run_cmd,cargo clippy,-p axlog $(1) $(verbose) -- $(clippy_args))
endef

all_packages := \
  $(shell ls $(CURDIR)/modules) \
  axfeat arceos_api axstd axlibc

define cargo_doc
  $(call run_cmd,cargo doc,--no-deps --all-features --workspace --exclude "arceos-*" $(verbose))
  @# run twice to fix broken hyperlinks
  $(foreach p,$(all_packages), \
    $(call run_cmd,cargo rustdoc,--all-features -p $(p) $(verbose))
  )
endef

define unit_test
  $(call run_cmd,cargo test,-p axfs $(1) $(verbose) -- --nocapture)
  $(call run_cmd,cargo test,-p axfs $(1) --features "myfs" $(verbose) -- --nocapture)
  $(call run_cmd,cargo test,--workspace --exclude axfs $(1) $(verbose) -- --nocapture)
endef
