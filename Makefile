# Arguments
ARCH ?= riscv64
MODE ?= release
LOG ?= warn
APP ?= helloworld
APP_LANG ?= rust

FS ?= off
NET ?= off

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
export MODE
export LOG

# Paths
app_package := arceos-$(APP)
kernel_elf := target/$(target)/$(MODE)/$(app_package)
kernel_bin := $(kernel_elf).bin

# Cargo features and build args

features := libax/platform-$(PLATFORM)

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

build_args := --no-default-features --features "$(features)" --target $(target) -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem
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
qemu_args := -nographic -m 128M

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

clean:
	cargo clean
	make -C ulib/c_libax clean

clippy:
	cargo clippy --target $(target)

fmt:
	cargo fmt --all

test:
	cargo test --workspace --exclude "arceos-*" --exclude "libax_bindings" -- --nocapture

.PHONY: build kernel_elf disasm run justrun clean clippy fmt test
