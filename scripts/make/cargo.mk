# Cargo features and build args

ifeq ($(V),1)
  verbose := -v
else ifeq ($(V),2)
  verbose := -vv
else
  verbose :=
endif

features-y :=

ifeq ($(shell test $(SMP) -gt 1; echo $$?),0)
  features-y += axstd/smp
  ifeq ($(APP_TYPE), c)
    features-y += axlibc/smp
  endif
endif

ifneq ($(filter $(LOG),off error warn info debug trace),)
  features-y += axstd/log-level-$(LOG)
else
  $(error "LOG" must be one of "off", "error", "warn", "info", "debug", "trace")
endif

ifeq ($(BUS),pci)
  features-y += axstd/bus-pci
endif

ifeq ($(APP_TYPE),c)
  ifneq ($(wildcard $(APP)/features.txt),)    # check features.txt exists
    features_c := $(shell cat $(APP)/features.txt)
    ifneq ($(strip $(foreach feat,fs net pipe select epoll,$(filter $(feat),$(features_c)))),)
      features_c += fd
    endif
    CFLAGS += $(addprefix -DAX_CONFIG_,$(shell echo $(features_c) | tr 'a-z' 'A-Z'))
  endif
  features-y += $(addprefix axlibc/,$(features_c))
  features-y += $(APP_FEATURES)
else ifeq ($(APP_TYPE),rust)
  features-y += $(APP_FEATURES)
endif

build_args-release := --release

build_args := \
  --target $(TARGET) \
  --target-dir $(CURDIR)/target \
  $(build_args-$(MODE)) \
  --features "$(features-y)" \
  $(verbose)

RUSTFLAGS := -C link-arg=-T$(LD_SCRIPT) -C link-arg=-no-pie
RUSTDOCFLAGS := --enable-index-page -Zunstable-options -D rustdoc::broken_intra_doc_links

ifeq ($(MAKECMDGOALS), doc_check_missing)
  RUSTDOCFLAGS += -D missing-docs
endif

define cargo_rustc
  $(call run_cmd,cargo rustc,$(build_args) $(1))
endef

define cargo_clippy
  $(call run_cmd,cargo clippy,--all-features --workspace --exclude axlog $(1) $(verbose))
  $(call run_cmd,cargo clippy,-p axlog -p percpu -p percpu_macros $(1) $(verbose))
endef

all_packages := \
  $(shell ls $(CURDIR)/crates) \
  $(shell ls $(CURDIR)/modules) \
  axstd axlibc

define cargo_doc
  $(call run_cmd,cargo doc,--no-deps --all-features --workspace --exclude "arceos-*" $(verbose))
  @# run twice to fix broken hyperlinks
  $(foreach p,$(all_packages), \
    $(call run_cmd,cargo rustdoc,--all-features -p $(p) $(verbose))
  )
  @# for some crates, re-generate without `--all-features`
  $(call run_cmd,cargo doc,--no-deps -p percpu $(verbose))
endef
