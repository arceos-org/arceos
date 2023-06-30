# Cargo features and build args

ifeq ($(V),1)
  verbose := -v
else ifeq ($(V),2)
  verbose := -vv
else
  verbose :=
endif

features-y := libax/platform-$(PLATFORM)

ifeq ($(shell test $(SMP) -gt 1; echo $$?),0)
  features-y += libax/smp
endif

ifneq ($(filter $(LOG),off error warn info debug trace),)
  features-y += libax/log-level-$(LOG)
else
  $(error "LOG" must be one of "off", "error", "warn", "info", "debug", "trace")
endif

features-$(FS) += libax/fs
features-$(NET) += libax/net
features-$(GRAPHIC) += libax/display

ifeq ($(BUS),pci)
  features-y += libax/bus-pci
endif

default_features := y

ifeq ($(APP_TYPE),c)
  default_features := n
  ifneq ($(wildcard $(APP)/features.txt),)    # check features.txt exists
    features-y += $(addprefix libax/,$(shell cat $(APP)/features.txt))
    CFLAGS += $(addprefix -DAX_CONFIG_,$(shell cat $(APP)/features.txt | tr 'a-z' 'A-Z'))
  endif
  features-y += libax/cbindings
  features-y += $(APP_FEATURES)
else ifeq ($(APP_TYPE),rust)
  features-y += $(APP_FEATURES)
  ifneq ($(APP_FEATURES),)
    default_features := n
  endif
endif

build_args-release := --release
build_args-c := --crate-type staticlib
build_args-rust :=

build_args := \
  --target $(TARGET) \
  --target-dir $(CURDIR)/target \
  $(build_args-$(MODE)) \
  $(build_args-$(APP_TYPE)) \
  --features "$(features-y)" \

ifeq ($(default_features),n)
  build_args += --no-default-features
endif

RUSTFLAGS := -C link-arg=-T$(LD_SCRIPT) -C link-arg=-no-pie

define cargo_rustc
  $(call run_cmd,cargo rustc,$(build_args) $(1) $(verbose) -- $(RUSTFLAGS))
endef

define cargo_clippy
  $(call run_cmd,cargo clippy,--target $(TARGET) --all-features --workspace --exclude axlog $(verbose))
  $(call run_cmd,cargo clippy,--target $(TARGET) -p axlog -p percpu -p percpu_macros $(verbose))
endef

all_packages := \
  $(shell ls $(CURDIR)/crates) \
  $(shell ls $(CURDIR)/modules) \
  libax

define cargo_doc
  RUSTDOCFLAGS="--enable-index-page -Zunstable-options -D rustdoc::broken_intra_doc_links $(1)" \
    cargo doc --no-deps --all-features --workspace --exclude "arceos-*" $(verbose)
  @# run twice to fix broken hyperlinks
  $(foreach p,$(all_packages), \
    $(call run_cmd,cargo rustdoc,--all-features -p $(p) $(verbose))
  )
  @# for some crates, re-generate without `--all-features`
  $(call run_cmd,cargo doc,--no-deps -p percpu $(verbose))
endef
