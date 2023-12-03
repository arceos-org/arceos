# Utility definitions and functions

GREEN_C := \033[92;1m
CYAN_C := \033[96;1m
YELLOW_C := \033[93;1m
GRAY_C := \033[90m
WHITE_C := \033[37m
END_C := \033[0m

define run_cmd
  @printf '$(WHITE_C)$(1)$(END_C) $(GRAY_C)$(2)$(END_C)\n'
  @$(1) $(2)
endef

define make_disk_image_fat32
  @printf "    $(GREEN_C)Creating$(END_C) FAT32 disk image \"$(1)\" ...\n"
  @dd if=/dev/zero of=$(1) bs=1M count=64
  @mkfs.fat -F 32 $(1)
endef

define make_disk_image
  $(if $(filter $(1),fat32), $(call make_disk_image_fat32,$(2)))
endef

# 用于宏内核启动时预先编译应用程序，当前仅支持riscv架构
define make_bin
  @printf "Make bin for Starry\n"
	@$(CC) -static $(A)/main.c -o testcases/sdcard/main
  @rust-objdump testcases/sdcard/main -S testcases/sdcard/main -l > $(A)/syscall.S
  @python $(APP)/build.py $(A)/syscall.S $(APP)/features.txt img
  @rm -rf $(A)/syscall.S
  @sh ./build_img.sh sdcard
endef
