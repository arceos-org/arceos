# Arguments
ARCH ?= x86_64
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
BUS ?= mmio

QEMU_LOG ?= n

ifeq ($(wildcard $(APP)),)
  $(error Application path "$(APP)" is not valid)
endif

ifneq ($(wildcard $(APP)/Cargo.toml),)
  APP_LANG ?= rust
else
  APP_LANG ?= c
endif

# Platform
ifeq ($(ARCH), x86_64)
  ACCEL ?= y
  PLATFORM ?= pc-x86
  TARGET := x86_64-unknown-none
  BUS := pci
else ifeq ($(ARCH), riscv64)
  ACCEL ?= n
  PLATFORM ?= qemu-virt-riscv
  TARGET := riscv64gc-unknown-none-elf
  TARGET_CFLAGS := -mabi=lp64d
else ifeq ($(ARCH), aarch64)
  ACCEL ?= n
  PLATFORM ?= qemu-virt-aarch64
  TARGET := aarch64-unknown-none-softfloat
else
  $(error "ARCH" must be one of "x86_64", "riscv64", or "aarch64")
endif

export ARCH
export PLATFORM
export SMP
export MODE
export LOG

# Binutils
CROSS_COMPILE ?= $(ARCH)-linux-musl-
CC := $(CROSS_COMPILE)gcc
AR := $(CROSS_COMPILE)ar
RANLIB := $(CROSS_COMPILE)ranlib
LD := rust-lld -flavor gnu

OBJDUMP ?= rust-objdump -d --print-imm-hex --x86-asm-syntax=intel
OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)
GDB ?= gdb-multiarch

export TARGET_CC = $(CC)
export TARGET_CFLAGS

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
	$(call run_qemu_debug) &
	sleep 1
	$(GDB) $(OUT_ELF) \
	  -ex 'target remote localhost:1234' \
	  -ex 'b rust_entry' \
	  -ex 'continue' \
	  -ex 'disp /16i $$pc'

clippy:
	$(call cargo_clippy)

doc:
	$(call cargo_doc)

doc_check_missing:
	$(call cargo_doc,-D missing-docs)

fmt:
	cargo fmt --all

fmt_c:
	@clang-format --style=file -i $(shell find ulib/c_libax -iname '*.c' -o -iname '*.h')

test:
	$(call app_test)

unittest:
	$(call unit_test)

unittest_no_fail_fast:
	$(call unit_test,--no-fail-fast)

disk_img:
ifneq ($(wildcard $(DISK_IMG)),)
	@printf "$(YELLOW_C)warning$(END_C): disk image \"$(DISK_IMG)\" already exists!\n"
else
	$(call make_disk_image,fat32,$(DISK_IMG))
endif

clean: clean_c
	rm -rf $(APP)/*.bin $(APP)/*.elf
	cargo clean

clean_c:
	rm -rf ulib/c_libax/build_*
	rm -rf $(app-objs)

.PHONY: all build disasm run justrun debug clippy fmt fmt_c test test_no_fail_fast clean clean_c doc disk_image
