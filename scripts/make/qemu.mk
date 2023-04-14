# QEMU arguments

QEMU := qemu-system-$(ARCH)

qemu_args-riscv64 := \
  -machine virt \
  -bios default \
  -kernel $(OUT_BIN)

qemu_args-aarch64 := \
  -cpu cortex-a72 \
  -machine virt \
  -kernel $(OUT_BIN)

qemu_args-y := -m 128M -smp $(SMP) $(qemu_args-$(ARCH))

qemu_args-$(FS) += \
  -device virtio-blk-device,drive=disk0 \
  -drive id=disk0,if=none,format=raw,file=$(DISK_IMG)

qemu_args-$(NET) += \
  -device virtio-net-device,netdev=net0 \
  -netdev user,id=net0,hostfwd=tcp::5555-:5555

qemu_args-$(GRAPHIC) += \
  -device virtio-gpu-device \
  -serial mon:stdio

ifeq ($(GRAPHIC), n)
  qemu_args-y += -nographic
endif

define run_qemu
  @echo "    $(CYAN_C)Running$(END_C) $(QEMU) $(qemu_args-y) $(1)"
  @$(QEMU) $(qemu_args-y) $(1)
endef
