# QEMU arguments

QEMU := qemu-system-$(ARCH)

qemu_args-x86_64 := \
  -machine q35 \
  -kernel $(OUT_ELF)

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
  -netdev user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555

qemu_args-$(GRAPHIC) += \
  -device virtio-gpu-device \
  -serial mon:stdio

ifeq ($(GRAPHIC), n)
  qemu_args-y += -nographic
endif

ifeq ($(QEMU_LOG), y)
  qemu_args-y += -D qemu.log -d in_asm,int,mmu,pcall,cpu_reset,guest_errors
endif

qemu_args-debug := $(qemu_args-y) -s -S

# Do not use KVM for debugging
ifeq ($(shell uname), Darwin)
  qemu_args-$(ACCEL) += -cpu host -accel hvf
else
  qemu_args-$(ACCEL) += -cpu host -accel kvm
endif

define run_qemu
  @printf "    $(CYAN_C)Running$(END_C) $(QEMU) $(qemu_args-y) $(1)\n"
  @$(QEMU) $(qemu_args-y)
endef

define run_qemu_debug
  @printf "    $(CYAN_C)Running$(END_C) $(QEMU) $(qemu_args-debug) $(1)\n"
  @$(QEMU) $(qemu_args-debug)
endef
