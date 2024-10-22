RK3588_GITHUB_URL = https://github.com/arceos-hypervisor/platform_tools/releases/download/latest/rk3588.zip
RK3588_MKIMG_FILE = ./tools/rk3588/mkimg
check-download:
ifeq ("$(wildcard $(RK3588_MKIMG_FILE))","")
		@echo "file not found, downloading from $(RK3588_GITHUB_URL)..."; 
		wget $(RK3588_GITHUB_URL); 
		unzip -o rk3588.zip -d tools; 
		rm rk3588.zip; 
endif

kernel: check-download build
	$(RK3588_MKIMG_FILE) --dtb rk3588-firefly-itx-3588j.dtb --img $(OUT_BIN)
	@echo 'Built the FIT-uImage boot.img'
