# Utility definitions and functions

GREEN_C := \033[92;1m
CYAN_C := \033[96;1m
YELLOW_C := \033[93;1m
END_C := \033[0m

define make_disk_image_fat32
  @printf "    $(GREEN_C)Creating$(END_C) FAT32 disk image \"$(1)\" ...\n"
  @dd if=/dev/zero of=$(1) bs=1M count=64
  @mkfs.fat -F 32 $(1)
endef

define make_disk_image_ext2
  @echo "    $(GREEN_C)Creating$(END_C) EXT2 disk image \"$(1)\" ..."
  @cd ext2fs_fuse && cargo run
endef

define make_disk_image
  $(if $(filter $(1),fat32), $(call make_disk_image_fat32,$(2)))
  $(if $(filter $(1),ext2), $(call make_disk_image_ext2))
endef
