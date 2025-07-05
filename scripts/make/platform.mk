# Architecture and platform resolving

ifeq ($(APP_TYPE), rust)
  cargo_manifest_dir := $(APP)
else
  cargo_manifest_dir := $(CURDIR)
endif

define resolve_config
  $(if $(wildcard $(PLAT_CONFIG)),\
    $(PLAT_CONFIG),\
    $(shell cargo axplat info -C $(cargo_manifest_dir) -c $(PLAT_PACKAGE)))
endef

define validate_config
  $(eval package := $(shell axconfig-gen $(PLAT_CONFIG) -r package 2>/dev/null)) \
  $(if $(strip $(package)),,$(error PLAT_CONFIG=$(PLAT_CONFIG) is not a valid platform configuration file)) \
  $(if $(filter "$(PLAT_PACKAGE)",$(package)),,\
    $(error `PLAT_PACKAGE` field mismatch: expected $(PLAT_PACKAGE), got $(package)))
endef

ifeq ($(MYPLAT),)
  # `MYPLAT` is not specified, use the default platform for each architecture
  ifeq ($(ARCH), x86_64)
    PLAT_PACKAGE := axplat-x86-pc
  else ifeq ($(ARCH), aarch64)
    PLAT_PACKAGE := axplat-aarch64-qemu-virt
  else ifeq ($(ARCH), riscv64)
    PLAT_PACKAGE := axplat-riscv64-qemu-virt
  else ifeq ($(ARCH), loongarch64)
    PLAT_PACKAGE := axplat-loongarch64-qemu-virt
  else
    $(error "ARCH" must be one of "x86_64", "riscv64", "aarch64" or "loongarch64")
  endif
  PLAT_CONFIG := $(strip $(call resolve_config))
  # We don't need to check whether `PLAT_CONFIG` is valid here, as the `PLAT_PACKAGE`
  # is a valid pacakage.

  $(call validate_config)
else
  # `MYPLAT` is specified, treat it as a package name
  PLAT_PACKAGE := $(MYPLAT)
  PLAT_CONFIG := $(strip $(call resolve_config))
  ifeq ($(wildcard $(PLAT_CONFIG)),)
    $(error "MYPLAT=$(MYPLAT) is not a valid platform package name")
  endif
  $(call validate_config)

  # Read the architecture name from the configuration file
  _arch := $(patsubst "%",%,$(shell axconfig-gen $(PLAT_CONFIG) -r arch))
  ifeq ($(origin ARCH),command line)
    ifneq ($(ARCH),$(_arch))
      $(error "ARCH=$(ARCH)" is not compatible with "MYPLAT=$(MYPLAT)")
    endif
  endif
  ARCH := $(_arch)
endif

PLAT_NAME := $(patsubst "%",%,$(shell axconfig-gen $(PLAT_CONFIG) -r platform))
