# Arguments
ARCH ?= riscv64
SMP ?= 1
MODE ?= release
LOG ?= warn

APP ?= helloworld
APP_LANG ?= rust
APP_FEATURES ?=

FS ?= off
NET ?= off
GRAPHIC ?= off

# Platform
ifeq ($(ARCH), riscv64)
  PLATFORM ?= qemu-virt-riscv
  target := riscv64gc-unknown-none-elf
else ifeq ($(ARCH), aarch64)
  PLATFORM ?= qemu-virt-aarch64
  target := aarch64-unknown-none-softfloat
else
  $(error "ARCH" must be "riscv64" or "aarch64")
endif

export ARCH
export PLATFORM
export SMP
export MODE
export LOG

# Paths
app_package := arceos-$(APP)
kernel_elf := target/$(target)/$(MODE)/$(app_package)
kernel_bin := $(kernel_elf).bin

# Cargo features and build args

features := $(APP_FEATURES) libax/platform-$(PLATFORM)

ifeq ($(shell test $(SMP) -gt 1; echo $$?),0)
  features += libax/smp
endif

ifneq ($(filter $(LOG),off error warn info debug trace),)
  features += libax/log-level-$(LOG)
else
  $(error "LOG" must be one of "off", "error", "warn", "info", "debug", "trace")
endif

ifeq ($(FS), on)
  features += libax/fs
endif
ifeq ($(NET), on)
  features += libax/net
endif
ifeq ($(GRAPHIC), on)
  features += libax/display
endif

build_args := --features "$(features)" --target $(target) -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem
ifneq ($(APP_FEATURES),)
  build_args += --no-default-features
endif
ifeq ($(MODE), release)
  build_args += --release
endif

build_args += -p $(app_package)

# Binutils
OBJDUMP := rust-objdump -d --print-imm-hex --x86-asm-syntax=intel
OBJCOPY := rust-objcopy --binary-architecture=$(ARCH)
GDB := gdb-multiarch

# QEMU
qemu := qemu-system-$(ARCH)
qemu_args := -m 128M  -smp $(SMP)

ifeq ($(ARCH), riscv64)
  qemu_args += \
    -machine virt \
    -bios default \
    -kernel $(kernel_bin)
else ifeq ($(ARCH), aarch64)
  qemu_args += \
    -cpu cortex-a72 \
    -machine virt \
    -kernel $(kernel_bin)
endif

ifeq ($(FS), on)
  qemu_args += \
    -device virtio-blk-device,drive=disk0 \
    -drive id=disk0,if=none,format=raw,file=disk.img
endif
ifeq ($(NET), on)
  qemu_args += \
    -device virtio-net-device,netdev=net0 \
    -netdev user,id=net0,hostfwd=tcp::5555-:5555
endif
ifeq ($(GRAPHIC), on)
  qemu_args += \
    -device virtio-gpu-device \
    -serial mon:stdio
else
  qemu_args += -nographic
endif

build: $(kernel_bin)

kernel_elf:
	@echo Arch: $(ARCH), Platform: $(PLATFORM), Language: $(APP_LANG)
ifeq ($(APP_LANG), rust)
	cargo build $(build_args)
else ifeq ($(APP_LANG), c)
	@rm -f $(kernel_elf)
	@make -C ulib/c_libax ARCH=$(ARCH) MODE=$(MODE) APP=$(APP) FEATURES="$(features)"
endif

$(kernel_bin): kernel_elf
	@$(OBJCOPY) $(kernel_elf) --strip-all -O binary $@

disasm:
	$(OBJDUMP) $(kernel_elf) | less

run: build justrun

justrun:
	$(qemu) $(qemu_args)

debug: build
	$(qemu) $(qemu_args) -s -S &
	sleep 1
	$(GDB) $(kernel_elf) -ex 'target remote localhost:1234'

clean:
	cargo clean
	make -C ulib/c_libax clean

clippy:
	cargo clippy --target $(target)

fmt:
	cargo fmt --all

test:
	cargo test --workspace --exclude "arceos-*" --exclude "libax_bindings" -- --nocapture

test_no_fail_fast:
	cargo test --workspace --exclude "arceos-*" --exclude "libax_bindings" --no-fail-fast -- --nocapture

.PHONY: build kernel_elf disasm run justrun clean clippy fmt test test_no_fail_fast
