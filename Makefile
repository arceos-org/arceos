# Arguments
ARCH ?= riscv64
SMP ?= 1
MODE ?= release
LOG ?= warn

A ?= apps/helloworld
APP ?= $(A)
APP_FEATURES ?=
DISK_IMG ?= disk.img

FS ?= n
NET ?= n
GRAPHIC ?= n

ifeq ($(wildcard $(APP)),)
  $(error Application path "$(APP)" is not valid)
endif

ifneq ($(wildcard $(APP)/Cargo.toml),)
  APP_LANG ?= rust
else
  APP_LANG ?= c
endif

# Platform
ifeq ($(ARCH), riscv64)
  PLATFORM ?= qemu-virt-riscv
  TARGET := riscv64gc-unknown-none-elf
else ifeq ($(ARCH), aarch64)
  PLATFORM ?= qemu-virt-aarch64
  TARGET := aarch64-unknown-none-softfloat
else
  $(error "ARCH" must be "riscv64" or "aarch64")
endif

export ARCH
export PLATFORM
export SMP
export MODE
export LOG

# Binutils
ifeq ($(APP_LANG), c)
  CROSS_COMPILE ?= $(ARCH)-linux-musl-
  CC := $(CROSS_COMPILE)gcc
  LD := $(CROSS_COMPILE)ld
  AR := $(CROSS_COMPILE)ar
  RANLIB := $(CROSS_COMPILE)ranlib
endif

OBJDUMP ?= rust-objdump -d --print-imm-hex --x86-asm-syntax=intel
OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)
GDB ?= gdb-multiarch

# Paths
OUT_DIR ?= $(APP)

APP_NAME := $(shell basename $(APP))
LD_SCRIPT := $(CURDIR)/modules/axhal/linker_$(ARCH).lds
OUT_ELF := $(OUT_DIR)/$(APP_NAME)_$(PLATFORM).elf
OUT_BIN := $(OUT_DIR)/$(APP_NAME)_$(PLATFORM).bin

all: build

include scripts/make/utils.mk
include scripts/make/cargo.mk
include scripts/make/qemu.mk
include scripts/make/build.mk
include scripts/make/test.mk

build: $(OUT_DIR) $(OUT_BIN)

disasm:
	$(OBJDUMP) $(OUT_ELF) | less

run: build justrun

justrun:
	$(call run_qemu)

debug: build
	$(call run_qemu,-s -S) &
	sleep 1
	$(GDB) $(OUT_ELF) -ex 'target remote localhost:1234'

clippy:
	cargo clippy --target $(TARGET)

doc:
	$(call cargo_doc)

fmt:
	cargo fmt --all

fmt_c:
	@clang-format --style=file -i $(shell find ulib/c_libax -iname '*.c' -o -iname '*.h')

test:
	$(call unittest)

test_no_fail_fast:
	$(call unittest,--no-fail-fast)

disk_image:
ifneq ($(wildcard $(DISK_IMG)),)
	@echo "$(YELLOW_C)warning$(END_C): image \"$(DISK_IMG)\" already exists!"
else
	$(call make_disk_image,fat32,$(DISK_IMG))
endif

clean: clean_c
	rm -rf $(APP)/*.bin $(APP)/*.elf
	cargo clean

clean_c:
	rm -rf ulib/c_libax/build_*
	rm -rf $(APP)/*.o

.PHONY: all build disasm run justrun debug clippy fmt fmt_c test test_no_fail_fast clean clean_c doc disk_image
