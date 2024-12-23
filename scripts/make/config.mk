# Config generation

config_file := .axconfig.toml

config_args := \
  configs/defconfig.toml configs/platforms/$(PLAT_NAME).toml \
  -w 'smp=$(SMP)' \
  -w 'platform.arch="$(ARCH)"' \
  -w 'platform.plat-name="$(PLAT_NAME)"' \
  -o $(config_file)

define defconfig
  $(call run_cmd,axconfig-gen,$(config_args))
endef

ifneq ($(wildcard $(config_file)),)
  define oldconfig
    $(if $(filter "$(PLAT_NAME)",$(shell axconfig-gen $(config_file) -r platform.plat-name)),\
         ,\
         $(error "ARCH" or "PLATFORM" has been changed, please run "make defconfig" again))
    $(call run_cmd,axconfig-gen,$(config_args) -c $(config_file))
  endef
else
  define oldconfig
    $(call defconfig)
  endef
endif
