# Architecture and platform resolving

# A map of builtin platforms to their corresponding package names
builtin_platforms_map := \
    x86_64-qemu-q35:x86-pc \
    x86_64-pc-oslab:x86-pc \
    riscv64-qemu-virt:riscv64-qemu-virt \
    aarch64-qemu-virt:aarch64-qemu-virt \
    aarch64-raspi4:aarch64-raspi \
    aarch64-bsta1000b:aarch64-bsta1000b \
    aarch64-phytium-pi:aarch64-phytium-pi \
    loongarch64-qemu-virt:loongarch64-qemu-virt

# Resolve the path of platform configuration file
define resolve_plat_config 
  $(addsuffix /axconfig.toml, \
    $(shell cargo metadata --all-features --format-version 1 | \
      jq '.packages[] | select(.name == "axplat-$(PLAT_PACKAGE)") | .manifest_path' | \
      xargs dirname))
endef
ifeq ($(PLATFORM),)
  # `PLATFORM` is not specified, use the default platform for each architecture
  ifeq ($(ARCH), x86_64)
    PLAT_NAME := x86_64-qemu-q35
    PLAT_PACKAGE := x86-pc
  else ifeq ($(ARCH), aarch64)
    PLAT_NAME := aarch64-qemu-virt
    PLAT_PACKAGE := aarch64-qemu-virt
  else ifeq ($(ARCH), riscv64)
    PLAT_NAME := riscv64-qemu-virt
    PLAT_PACKAGE := riscv64-qemu-virt
  else ifeq ($(ARCH), loongarch64)
    PLAT_NAME := loongarch64-qemu-virt
    PLAT_PACKAGE := loongarch64-qemu-virt
  else
    $(error "ARCH" must be one of "x86_64", "riscv64", "aarch64" or "loongarch64")
  endif
  PLAT_CONFIG := $(call resolve_plat_config)
else
  platform_pair = $(filter $(PLATFORM):%, $(builtin_platforms_map))
  # `PLATFORM` is specified, override the `ARCH` variables
  ifneq ($(platform_pair),)
    # builtin platform
    _arch := $(word 1,$(subst -, ,$(PLATFORM)))
    PLAT_NAME := $(PLATFORM)
    PLAT_PACKAGE := $(word 2, $(subst :, ,$(platform_pair)))
    PLAT_CONFIG := $(call resolve_plat_config)
  else ifneq ($(wildcard $(PLATFORM)),)
    # custom platform, read the "arch" and "plat-name" fields from the toml file
    _arch :=  $(patsubst "%",%,$(shell axconfig-gen $(PLATFORM) -r arch))
    PLAT_NAME := $(patsubst "%",%,$(shell axconfig-gen $(PLATFORM) -r platform))
    PLAT_CONFIG := $(PLATFORM)
  else
    builtin_platforms := $(foreach pair,$(builtin_platforms_map),$(firstword $(subst :, ,$(pair))))
    $(error "PLATFORM" must be one of "$(builtin_platforms)" or a valid path to a toml file)
  endif
  ifeq ($(origin ARCH),command line)
    ifneq ($(ARCH),$(_arch))
      $(error "ARCH=$(ARCH)" is not compatible with "PLATFORM=$(PLATFORM)")
    endif
  endif
  ARCH := $(_arch)
endif
