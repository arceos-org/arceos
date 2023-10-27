# Available arguments:
# * General options:
#     - `ARCH`: Target architecture: x86_64, riscv64, aarch64
#     - `PLATFORM`: Target platform in the `platforms` directory
#     - `SMP`: Number of CPUs
#     - `MODE`: Build mode: release, debug
#     - `LOG:` Logging level: warn, error, info, debug, trace
#     - `V`: Verbose level: (empty), 1, 2
# * App options:
#     - `A` or `APP`: Path to the application
#     - `FEATURES`: Features os ArceOS modules to be enabled.
#     - `APP_FEATURES`: Features of (rust) apps to be enabled.
# * QEMU options:
#     - `BLK`: Enable storage devices (virtio-blk)
#     - `NET`: Enable network devices (virtio-net)
#     - `GRAPHIC`: Enable display devices and graphic output (virtio-gpu)
#     - `BUS`: Device bus type: mmio, pci
#     - `DISK_IMG`: Path to the virtual disk image
#     - `ACCEL`: Enable hardware acceleration (KVM on linux)
#     - `QEMU_LOG`: Enable QEMU logging (log file is "qemu.log")
#     - `NET_DUMP`: Enable network packet dump (log file is "netdump.pcap")
#     - `NET_DEV`: QEMU netdev backend types: user, tap, bridge
#     - `VFIO_PCI`: PCI device address in the format "bus:dev.func" to passthrough
#     - `VHOST`: Enable vhost-net for tap backend (only for `NET_DEV=tap`)
# * Network options:
#     - `IP`: ArceOS IPv4 address (default is 10.0.2.15 for QEMU user netdev)
#     - `GW`: Gateway IPv4 address (default is 10.0.2.2 for QEMU user netdev)

# General options
ARCH ?= x86_64
PLATFORM ?=
SMP ?= 1
MODE ?= release
LOG ?= warn
V ?=

# App options
A ?= apps/helloworld
APP ?= $(A)
FEATURES ?=
APP_FEATURES ?=

# QEMU options
BLK ?= n
NET ?= n
GRAPHIC ?= n
BUS ?= mmio

DISK_IMG ?= disk.img
QEMU_LOG ?= n
NET_DUMP ?= n
NET_DEV ?= user
VFIO_PCI ?=
VHOST ?= n

# Network options
IP ?= 10.0.2.15
GW ?= 10.0.2.2

# App type
ifeq ($(wildcard $(APP)),)
  $(error Application path "$(APP)" is not valid)
endif

ifneq ($(wildcard $(APP)/Cargo.toml),)
  APP_TYPE := rust
else
  APP_TYPE := c
endif

# Architecture, platform and target
ifneq ($(filter $(MAKECMDGOALS),unittest unittest_no_fail_fast),)
  PLATFORM_NAME :=
else ifneq ($(PLATFORM),)
  # `PLATFORM` is specified, override the `ARCH` variables
  builtin_platforms := $(patsubst platforms/%.toml,%,$(wildcard platforms/*))
  ifneq ($(filter $(PLATFORM),$(builtin_platforms)),)
    # builtin platform
    PLATFORM_NAME := $(PLATFORM)
    _arch := $(word 1,$(subst -, ,$(PLATFORM)))
  else ifneq ($(wildcard $(PLATFORM)),)
    # custom platform, read the "platform" field from the toml file
    PLATFORM_NAME := $(shell cat $(PLATFORM) | sed -n 's/^platform = "\([a-z0-9A-Z_\-]*\)"/\1/p')
    _arch := $(shell cat $(PLATFORM) | sed -n 's/^arch = "\([a-z0-9A-Z_\-]*\)"/\1/p')
  else
    $(error "PLATFORM" must be one of "$(builtin_platforms)" or a valid path to a toml file)
  endif
  ifeq ($(origin ARCH),command line)
    ifneq ($(ARCH),$(_arch))
      $(error "ARCH=$(ARCH)" is not compatible with "PLATFORM=$(PLATFORM)")
    endif
  endif
  ARCH := $(_arch)
endif

ifeq ($(ARCH), x86_64)
  # Don't enable kvm for WSL/WSL2.
  ACCEL ?= $(if $(findstring -microsoft, $(shell uname -r | tr '[:upper:]' '[:lower:]')),n,y)
  PLATFORM_NAME ?= x86_64-qemu-q35
  TARGET := x86_64-unknown-none
  BUS := pci
else ifeq ($(ARCH), riscv64)
  ACCEL ?= n
  PLATFORM_NAME ?= riscv64-qemu-virt
  TARGET := riscv64gc-unknown-none-elf
else ifeq ($(ARCH), aarch64)
  ACCEL ?= n
  PLATFORM_NAME ?= aarch64-qemu-virt
  TARGET := aarch64-unknown-none-softfloat
else
  $(error "ARCH" must be one of "x86_64", "riscv64", or "aarch64")
endif

export AX_ARCH=$(ARCH)
export AX_PLATFORM=$(PLATFORM_NAME)
export AX_SMP=$(SMP)
export AX_MODE=$(MODE)
export AX_LOG=$(LOG)
export AX_TARGET=$(TARGET)
export AX_IP=$(IP)
export AX_GW=$(GW)

# Binutils
CROSS_COMPILE ?= $(ARCH)-linux-musl-
CC := $(CROSS_COMPILE)gcc
AR := $(CROSS_COMPILE)ar
RANLIB := $(CROSS_COMPILE)ranlib
LD := rust-lld -flavor gnu

OBJDUMP ?= rust-objdump -d --print-imm-hex --x86-asm-syntax=intel
OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)
GDB ?= gdb-multiarch

# Paths
OUT_DIR ?= $(APP)

APP_NAME := $(shell basename $(APP))
LD_SCRIPT := $(CURDIR)/modules/axhal/linker_$(PLATFORM_NAME).lds
OUT_ELF := $(OUT_DIR)/$(APP_NAME)_$(PLATFORM_NAME).elf
OUT_BIN := $(OUT_DIR)/$(APP_NAME)_$(PLATFORM_NAME).bin

all: build

include scripts/make/utils.mk
include scripts/make/build.mk
include scripts/make/qemu.mk
include scripts/make/test.mk
ifeq ($(PLATFORM_NAME), aarch64-raspi4)
  include scripts/make/raspi4.mk
else ifeq ($(PLATFORM_NAME), aarch64-bsta1000b)
  include scripts/make/bsta1000b-fada.mk
endif

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
ifeq ($(origin ARCH), command line)
	$(call cargo_clippy,--target $(TARGET))
else
	$(call cargo_clippy)
endif

doc:
	$(call cargo_doc)

doc_check_missing:
	$(call cargo_doc)

fmt:
	cargo fmt --all

fmt_c:
	@clang-format --style=file -i $(shell find ulib/axlibc -iname '*.c' -o -iname '*.h')

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

clean_c::
	rm -rf ulib/axlibc/build_*
	rm -rf $(app-objs)

.PHONY: all build disasm run justrun debug clippy fmt fmt_c test test_no_fail_fast clean clean_c doc disk_image
