ifeq ($(PLATFORM), aarch64-phytium-pi)
_uboot_img := arceos-phtpi.uImage
_kernel_base := $(subst _,,$(shell axconfig-gen configs/platforms/$(PLATFORM).toml -r plat.kernel-base-paddr))
phtpi: build
	@echo 'Create legacy uboot image for PhytiumPi: $(_uboot_img)'
	mkimage -A arm64 -O linux -C none -T kernel -a $(_kernel_base) -e $(_kernel_base) -n "ArceOS for PhytiumPi" -d $(OUT_BIN) $(_uboot_img)
	@echo 'Please boot from uboot> tftpboot $(_kernel_base) $(_uboot_img); bootm $(_kernel_base) - $${fdtcontroladdr}'
endif
