rust_lib_name := axlibc
rust_lib := target/$(TARGET)/$(MODE)/lib$(rust_lib_name).a

ulib_dir := ulib/axlibc
src_dir := $(ulib_dir)/c
obj_dir := $(ulib_dir)/build_$(ARCH)
inc_dir := $(ulib_dir)/include
inc_gen_dir = $(ulib_dir)/include_gen
c_lib := $(obj_dir)/libc.a
libgcc :=

last_cflags := $(obj_dir)/.cflags

ulib_src := $(wildcard $(src_dir)/*.c)
ulib_hdr := $(wildcard $(inc_dir)/*.h)
ulib_obj := $(patsubst $(src_dir)/%.c,$(obj_dir)/%.o,$(ulib_src))

CFLAGS += $(addprefix -DAX_CONFIG_,$(shell echo $(lib_feat) | tr 'a-z' 'A-Z' | tr '-' '_'))
CFLAGS += -DAX_LOG_$(shell echo $(LOG) | tr 'a-z' 'A-Z')

CFLAGS += -nostdinc -static -no-pie -fno-builtin -ffreestanding -Wall
CFLAGS += -I$(CURDIR)/$(inc_dir)
LDFLAGS += -nostdlib -static -no-pie --gc-sections -T$(LD_SCRIPT)

ifeq ($(MODE), release)
  CFLAGS += -O3
endif

ifeq ($(ARCH), riscv64)
  CFLAGS += -march=rv64gc -mabi=lp64d -mcmodel=medany
endif

ifeq ($(findstring fp_simd,$(FEATURES)),)
  ifeq ($(ARCH), x86_64)
    CFLAGS += -mno-sse
  else ifeq ($(ARCH), aarch64)
    CFLAGS += -mgeneral-regs-only
  endif
else
  ifeq ($(ARCH), riscv64)
    # for compiler-rt fallbacks like `__addtf3`, `__multf3`, ...
    libgcc := $(shell $(CC) -print-libgcc-file-name)
  endif
endif

_check_need_rebuild: $(obj_dir)
	@if [ "$(CFLAGS)" != "`cat $(last_cflags) 2>&1`" ]; then \
		echo "CFLAGS changed, rebuild"; \
		echo "$(CFLAGS)" > $(last_cflags); \
	fi

$(obj_dir):
	$(call run_cmd,mkdir,-p $@)

$(obj_dir)/%.o: $(src_dir)/%.c $(last_cflags)
	$(call run_cmd,$(CC),$(CFLAGS) -I$(inc_gen_dir) -c -o $@ $<)

$(c_lib): $(obj_dir) _check_need_rebuild $(ulib_obj)
	$(call run_cmd,$(AR),rcs $@ $(ulib_obj))

app-objs := main.o

-include $(APP)/axbuild.mk  # override `app-objs`

app-objs := $(addprefix $(APP)/,$(app-objs))

$(APP)/%.o: $(APP)/%.c $(ulib_hdr)
	$(call run_cmd,$(CC),$(CFLAGS) $(APP_CFLAGS) -c -o $@ $<)

$(OUT_ELF): $(c_lib) $(rust_lib) $(libgcc) $(app-objs)
	@printf "    $(CYAN_C)Linking$(END_C) $(OUT_ELF)\n"
	$(call run_cmd,$(LD),$(LDFLAGS) $^ -o $@)

$(APP)/axbuild.mk: ;

.PHONY: _check_need_rebuild
