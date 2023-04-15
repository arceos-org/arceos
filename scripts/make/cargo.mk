# Cargo features and build args

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

default_features := y

ifeq ($(APP_LANG),c)
  default_features := n
  ifneq ($(wildcard $(APP)/features.txt),)    # check features.txt exists
    features-y += $(addprefix libax/,$(shell cat $(APP)/features.txt))
    CFLAGS += $(addprefix -DAX_CONFIG_,$(shell cat $(APP)/features.txt | tr 'a-z' 'A-Z'))
  endif
  features-y += libax/cbindings
else ifeq ($(APP_LANG),rust)
  features-y += $(APP_FEATURES)
  ifneq ($(APP_FEATURES),)
    default_features := n
  endif
endif

build_args-release := --release
build_args-c := --crate-type staticlib
build_args-rust :=

build_args := \
  -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem \
  --target $(TARGET) \
  --target-dir $(CURDIR)/target \
  $(build_args-$(MODE)) \
  $(build_args-$(APP_LANG)) \
  --features "$(features-y)" \

ifeq ($(default_features),n)
  build_args += --no-default-features
endif

rustc_flags := -Clink-arg=-T$(LD_SCRIPT)

define cargo_build
  cargo rustc $(build_args) $(1) -- $(rustc_flags)
endef

define cargo_clippy
  cargo clippy --target $(TARGET) --all-features --workspace --exclude axlog
  cargo clippy --target $(TARGET) -p axlog -p percpu -p percpu_macros
endef

all_packages := \
  $(shell ls $(CURDIR)/crates) \
  $(shell ls $(CURDIR)/modules) \
  libax

define cargo_doc
  RUSTDOCFLAGS="--enable-index-page -Zunstable-options" cargo doc --no-deps --all-features --workspace --exclude "arceos-*"
  @# run twice to fix broken hyperlinks
  $(foreach p,$(all_packages), \
     cargo rustdoc --all-features -p $(p)
  )
  @# for some crates, re-generate without `--all-features`
  cargo doc --no-deps -p percpu -p kernel_guard
endef
