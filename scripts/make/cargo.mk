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
    features-y += $(addprefix libax_bindings/,$(shell cat $(APP)/features.txt))
    CFLAGS += $(addprefix -DAX_CONFIG_,$(shell cat $(APP)/features.txt | tr 'a-z' 'A-Z'))
  endif
else ifeq ($(APP_LANG),rust)
  features-y += $(APP_FEATURES)
  ifneq ($(APP_FEATURES),)
    default_features := n
  endif
endif

build_args := \
  -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem \
  --config "build.rustflags='-Clink-arg=-T$(LD_SCRIPT)'" \
  --target $(TARGET) \
  --target-dir $(CURDIR)/target \
  --features "$(features-y)" \

ifeq ($(default_features),n)
  build_args += --no-default-features
endif

ifeq ($(MODE), release)
  build_args += --release
endif

define cargo_build
  cargo build $(build_args) $(1)
endef

define cargo_doc
  RUSTDOCFLAGS="--enable-index-page -Zunstable-options" \
  cargo doc --no-deps --target $(TARGET) --workspace --exclude "arceos-*"
endef
