kernel: build
	./tools/rk3588/mkimg --dtb rk3588-firefly-itx-3588j.dtb --img $(OUT_BIN)
	@echo 'Built the FIT-uImage boot.img'
