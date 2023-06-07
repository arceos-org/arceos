# Cargo features and build args
# 
# Modification for microkernel version
# 
# NOTE: this is only used for build and run
# for clippy, test, and doc, see original version `cargo.mk`
# (which has also been modified according to infrastructure changes)

features-kern-y := platform-$(PLATFORM)
features-user-y := 

# TODO: no SMP support is tested for microkernel
ifeq ($(shell test $(SMP) -gt 1; echo $$?),0)
  #features-y += libax/smp
endif

ifneq ($(filter $(KERN_LOG),off error warn info debug trace),)
  features-kern-y += log-level-$(KERN_LOG)
else
  $(error "KERN_LOG" must be one of "off", "error", "warn", "info", "debug", "trace")
endif

ifneq ($(filter $(USER_LOG),off error warn info debug trace),)
  features-user-y += libax/log-level-$(USER_LOG)
else
  $(error "USER_LOG" must be one of "off", "error", "warn", "info", "debug", "trace")
endif

features-kern-$(FS) += user-fs
features-kern-$(NET) += user-net
# features-$(GRAPHIC) += libax/display

# ifeq ($(BUS),pci)
#  features-y += libax/bus-pci
# endif

default_features := y

ifeq ($(APP_LANG),c)
  #default_features := n
  #ifneq ($(wildcard $(APP)/features.txt),)    # check features.txt exists
  #  features-y += $(addprefix libax/,$(shell cat $(APP)/features.txt))
  #  CFLAGS += $(addprefix -DAX_CONFIG_,$(shell cat $(APP)/features.txt | tr 'a-z' 'A-Z'))
  #endif
  #features-y += libax/cbindings
  #features-y += $(APP_FEATURES)
else ifeq ($(APP_LANG),rust)
  features-user-y += $(APP_FEATURES)
  ifneq ($(APP_FEATURES),)
    default_features := n
  endif
endif

build_args-release := --release
build_args-c := --crate-type staticlib
build_args-rust :=

build_args_kern := \
  --target $(TARGET) \
  --target-dir $(CURDIR)/target \
  $(build_args-$(MODE)) \
  $(build_args-$(APP_LANG)) \
  --features "$(features-kern-y)" \

build_args_user := \
  --target $(TARGET) \
  --target-dir $(CURDIR)/target \
  $(build_args-$(MODE)) \
  $(build_args-$(APP_LANG)) \
  --features "$(features-user-y)"

ifeq ($(default_features),n)
  build_args_user += --no-default-features
endif

LD_SCRIPT_USER := $(CURDIR)/ulib/libax_user/linker_$(ARCH).lds

rustc_flags_kern := -Clink-args="-T$(LD_SCRIPT) -no-pie"
rustc_flags_user := -Clink-args="-T$(LD_SCRIPT_USER) -no-pie"

define cargo_build_kern
  # must override
  LOG=$(KERN_LOG) cargo rustc $(build_args_kern) $(1) -- $(rustc_flags_kern)
endef

define cargo_build_user
  LOG=$(USER_LOG) cargo rustc $(build_args_user) $(1) -- $(rustc_flags_user)
endef


