# Arguments
ARCH ?= riscv64
MODE ?= release
LOG ?= warn
APP ?= helloworld

# Platform
ifeq ($(ARCH), riscv64)
  PLATFORM ?= qemu-virt-riscv
  target := riscv64gc-unknown-none-elf
endif

export ARCH
export PLATFORM
export MODE
export LOG

# Paths
kernel_package := arceos-$(APP)
kernel_elf := target/$(target)/$(MODE)/$(kernel_package)
kernel_bin := $(kernel_elf).bin

# Cargo features and build args

features := axhal/platform-$(PLATFORM)

build_args := --no-default-features --features "$(features)" --target $(target) -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem
ifeq ($(MODE), release)
  build_args += --release
endif

build_args += -p axruntime -p $(kernel_package)

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
endif

build: $(kernel_bin)

kernel_elf:
	@echo Arch: $(ARCH), Platform: $(PLATFORM)
	cargo build $(build_args)

$(kernel_bin): kernel_elf
	@$(OBJCOPY) $(kernel_elf) --strip-all -O binary $@

clean:
	cargo clean

clippy:
	cargo clippy $(build_args)

fmt:
	cargo fmt --all

disasm:
	$(OBJDUMP) $(kernel_elf) | less

run: build justrun

justrun:
	$(qemu) $(qemu_args)

.PHONY: build kernel_elf clean clippy fmt disasm run justrun
