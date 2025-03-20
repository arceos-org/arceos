# Available arguments:
# * General options:
#     - `ARCH`: Target architecture: x86_64, riscv64, aarch64
#     - `PLATFORM`: Target platform in the `platforms` directory
#     - `SMP`: Number of CPUs
#     - `MODE`: Build mode: release, debug
#     - `LOG:` Logging level: warn, error, info, debug, trace
#     - `V`: Verbose level: (empty), 1, 2
#     - `TARGET_DIR`: Artifact output directory (cargo target directory)
#     - `EXTRA_CONFIG`: Extra config specification file
#     - `OUT_CONFIG`: Final config file that takes effect
#     - `UIMAGE`: To generate U-Boot image
# * App options:
#     - `A` or `APP`: Path to the application
#     - `FEATURES`: Features os ArceOS modules to be enabled.
#     - `APP_FEATURES`: Features of (rust) apps to be enabled.
# * QEMU options:
#     - `BLK`: Enable storage devices (virtio-blk)
#     - `NET`: Enable network devices (virtio-net)
#     - `GRAPHIC`: Enable display devices and graphic output (virtio-gpu)
#     - `BUS`: Device bus type: mmio, pci
#     - `MEM`: Memory size (default is 128M)
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
TARGET_DIR ?= $(PWD)/target
EXTRA_CONFIG ?=
OUT_CONFIG ?= $(PWD)/.axconfig.toml
UIMAGE ?= n

# App options
A ?= examples/helloworld
APP ?= $(A)
FEATURES ?=
APP_FEATURES ?=

# QEMU options
BLK ?= n
NET ?= n
GRAPHIC ?= n
BUS ?= pci
MEM ?= 128M
ACCEL ?=

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
  AX_LIB ?= axstd
else
  APP_TYPE := c
  AX_LIB ?= axlibc
endif

NO_AXSTD ?= n

# Feature parsing
include scripts/make/features.mk
# Platform resolving
include scripts/make/platform.mk

# Target
ifeq ($(ARCH), x86_64)
  TARGET := x86_64-unknown-none
else ifeq ($(ARCH), aarch64)
  ifeq ($(findstring fp_simd,$(FEATURES)),)
    TARGET := aarch64-unknown-none-softfloat
  else
    TARGET := aarch64-unknown-none
  endif
else ifeq ($(ARCH), riscv64)
  TARGET := riscv64gc-unknown-none-elf
else ifeq ($(ARCH), loongarch64)
  TARGET := loongarch64-unknown-none
else
  $(error "ARCH" must be one of "x86_64", "riscv64", "aarch64" or "loongarch64")
endif

export AX_ARCH=$(ARCH)
export AX_PLATFORM=$(PLAT_NAME)
export AX_SMP=$(SMP)
export AX_MODE=$(MODE)
export AX_LOG=$(LOG)
export AX_TARGET=$(TARGET)
export AX_IP=$(IP)
export AX_GW=$(GW)

ifneq ($(filter $(MAKECMDGOALS),unittest unittest_no_fail_fast),)
  # When running unit tests, set `AX_CONFIG_PATH` to empty for dummy config
  unexport AX_CONFIG_PATH
else
  export AX_CONFIG_PATH=$(OUT_CONFIG)
endif

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
LD_SCRIPT := $(TARGET_DIR)/$(TARGET)/$(MODE)/linker_$(PLAT_NAME).lds
OUT_ELF := $(OUT_DIR)/$(APP_NAME)_$(PLAT_NAME).elf
OUT_BIN := $(patsubst %.elf,%.bin,$(OUT_ELF))
OUT_UIMG := $(patsubst %.elf,%.uimg,$(OUT_ELF))
ifeq ($(UIMAGE), y)
  FINAL_IMG := $(OUT_UIMG)
else
  FINAL_IMG := $(OUT_BIN)
endif

all: build

include scripts/make/utils.mk
include scripts/make/config.mk
include scripts/make/build.mk
include scripts/make/qemu.mk
ifeq ($(PLAT_NAME), aarch64-raspi4)
  include scripts/make/raspi4.mk
else ifeq ($(PLAT_NAME), aarch64-bsta1000b)
  include scripts/make/bsta1000b-fada.mk
endif

defconfig: _axconfig-gen
	$(call defconfig)

oldconfig: _axconfig-gen
	$(call oldconfig)

build: $(OUT_DIR) $(FINAL_IMG)

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

clippy: oldconfig
ifeq ($(origin ARCH), command line)
	$(call cargo_clippy,--target $(TARGET))
else
	$(call cargo_clippy)
endif

doc: oldconfig
	$(call cargo_doc)

doc_check_missing: oldconfig
	$(call cargo_doc)

fmt:
	cargo fmt --all

fmt_c:
	@clang-format --style=file -i $(shell find ulib/axlibc -iname '*.c' -o -iname '*.h')

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
	rm -rf $(APP)/*.bin $(APP)/*.elf $(OUT_CONFIG)
	cargo clean

clean_c::
	rm -rf ulib/axlibc/build_*
	rm -rf $(app-objs)

.PHONY: all defconfig oldconfig \
	build disasm run justrun debug \
	clippy doc doc_check_missing fmt fmt_c unittest unittest_no_fail_fast \
	disk_img clean clean_c
