# Config generation

config_args := \
  configs/defconfig.toml $(PLAT_CONFIG) $(EXTRA_CONFIG) \
  -w 'arch="$(ARCH)"' \
  -w 'platform="$(PLAT_NAME)"' \
  -o "$(OUT_CONFIG)"

ifneq ($(SMP),)
  config_args += -w 'plat.max-cpu-num=$(SMP)'
else
  SMP := $(shell axconfig-gen $(PLAT_CONFIG) -r plat.max-cpu-num 2>/dev/null)
  ifeq ($(SMP),)
    $(error "`plat.max-cpu-num` is not defined in the platform configuration file, \
      this option must be specified even for platforms with runtime CPU detection.")
  endif
endif

define defconfig
  $(call run_cmd,axconfig-gen,$(config_args))
endef

ifeq ($(wildcard $(OUT_CONFIG)),)
  define oldconfig
    $(call defconfig)
  endef
else
  define oldconfig
    $(if $(filter "$(PLAT_NAME)",$(shell axconfig-gen "$(OUT_CONFIG)" -r platform)),\
         $(call run_cmd,axconfig-gen,$(config_args) -c "$(OUT_CONFIG)"),\
         $(error "ARCH" or "MYPLAT" has been changed, please run "make defconfig" again))
  endef
endif

