# Arguments
ARCH ?= riscv64
MODE ?= release
FEATURES ?=
APP ?=
CROSS_COMPILE ?= $(ARCH)-linux-musl-

ifeq ($(ARCH), riscv64)
  target := riscv64gc-unknown-none-elf
else ifeq ($(ARCH), aarch64)
  target := aarch64-unknown-none-softfloat
else
  $(error "ARCH" must be "riscv64" or "aarch64")
endif

# Variables for Rust
app_path := ../../apps/c/$(APP)
rust_lib_name := libax_bindings
rust_build_path := ../../target/$(target)/$(MODE)
rust_lib := $(rust_build_path)/lib$(rust_lib_name).a

features := $(FEATURES)
ifneq (,$(wildcard $(app_path)/features.txt))	# check features.txt exists
  features += $(addprefix libax/,$(shell cat $(app_path)/features.txt))
endif

crago_build_args := --no-default-features --features "$(features)" --target $(target) -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem
ifeq ($(MODE), release)
  crago_build_args += --release
endif

# Variables for C
src_dir := src
obj_dir := build_$(ARCH)
inc_dir := include
c_lib := $(obj_dir)/libc.a

all_src := $(wildcard $(src_dir)/*.c)
all_obj := $(patsubst $(src_dir)/%.c,$(obj_dir)/%.o,$(all_src))

CC := $(CROSS_COMPILE)gcc
LD := $(CROSS_COMPILE)ld
AR := $(CROSS_COMPILE)ar
RANLIB := $(CROSS_COMPILE)ranlib
CFLAGS := -static -no-pie -fno-builtin -ffreestanding -nostdinc -Wall -I$(inc_dir) -I$(rust_lib_name)
LDFLAGS := -nostdlib -T../../modules/axhal/linker_$(ARCH).lds

ifeq ($(MODE), release)
  CFLAGS += -O3
endif

ifeq ($(ARCH), riscv64)
  CFLAGS += -march=rv64gc -mabi=lp64d -mcmodel=medany
else ifeq ($(ARCH), aarch64)
  CFLAGS += -mgeneral-regs-only
endif

# Variables for app
app_package := arceos-$(APP)
app_src := $(app_path)/main.c
app_obj := $(rust_build_path)/$(app_package).o
app_elf := $(rust_build_path)/$(app_package)

# Build commands
all: rust_lib c_lib app

rust_lib:
	@echo
	@echo Building rust static library $(rust_lib_name) ...
	cargo build $(crago_build_args) -p $(rust_lib_name)

c_lib: rust_lib $(obj_dir) $(c_lib)

$(obj_dir):
	@echo
	@echo Building C static library $(c_lib) ...
	mkdir -p $@

$(obj_dir)/%.o: $(src_dir)/%.c
	$(CC) $(CFLAGS) -c -o $@ $<

$(c_lib): $(all_obj)
	rm -f $@
	$(AR) rc $@ $(all_obj)
	$(RANLIB) $@

ifneq ($(APP),)
app: $(app_elf)

$(app_obj): $(app_src)
	$(CC) $(CFLAGS) -c -o $@ $<

$(app_elf): $(app_obj) $(c_lib) $(rust_lib)
	@echo
	@echo Building app \"$(app_package)\" ...
	$(LD) $(LDFLAGS) $^ -o $@
else
app:
endif

clean:
	rm -rf build_*
	cargo clean -p $(rust_lib_name)

.PHONY: all c_lib rust_lib app clean
