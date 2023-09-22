lpi4a: build
	mkimage -A riscv -O linux -C none -T kernel -a 0x200000 -e 0x200000 -n "ArceOS for THead light-lpi4a" \
		-d $(OUT_BIN) arceos-lpi4a.uImage
	@echo 'Built the u-boot image arceos-lpi4a.uImage'
