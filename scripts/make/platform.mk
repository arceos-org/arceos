# Architecture and platform resolving

ifneq ($(filter $(MAKECMDGOALS),unittest unittest_no_fail_fast),)
  PLAT_NAME :=
else ifeq ($(PLATFORM),)
  # `PLATFORM` is not specified, use the default platform for each architecture
  ifeq ($(ARCH), x86_64)
    PLAT_NAME := x86_64-qemu-q35
  else ifeq ($(ARCH), aarch64)
    PLAT_NAME := aarch64-qemu-virt
  else ifeq ($(ARCH), riscv64)
    PLAT_NAME := riscv64-qemu-virt
  endif
else ifneq ($(PLATFORM),)
  # `PLATFORM` is specified, override the `ARCH` variables
  builtin_platforms := $(patsubst configs/platforms/%.toml,%,$(wildcard configs/platforms/*))
  ifneq ($(filter $(PLATFORM),$(builtin_platforms)),)
    # builtin platform
    PLAT_NAME := $(PLATFORM)
    _arch := $(word 1,$(subst -, ,$(PLATFORM)))
  else ifneq ($(wildcard $(PLATFORM)),)
    # custom platform, read the "arch" and "plat-name" fields from the toml file
    PLAT_NAME := $(patsubst "%",%,$(shell axconfig-gen $(PLATFORM) -r platform.plat-name))
    _arch :=  $(patsubst "%",%,$(shell axconfig-gen $(PLATFORM) -r platform.arch))
  else
    $(error "PLATFORM" must be one of "$(builtin_platforms)" or a valid path to a toml file)
  endif
  ifeq ($(origin ARCH),command line)
    ifneq ($(ARCH),$(_arch))
      $(error "ARCH=$(ARCH)" is not compatible with "PLATFORM=$(PLATFORM)")
    endif
  endif
  ARCH := $(_arch)
endif
