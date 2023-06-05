rust_lib_name := libax
rust_lib := target/$(TARGET)/$(MODE)/lib$(rust_lib_name).a

ulib_dir := ulib/c_libax
src_dir := $(ulib_dir)/src
obj_dir := $(ulib_dir)/build_$(ARCH)
inc_dir := $(ulib_dir)/include
c_lib := $(obj_dir)/libc.a

in_feat := $(APP)/features.txt
out_feat := $(obj_dir)/.features.txt

ulib_src := $(wildcard $(src_dir)/*.c)
ulib_obj := $(patsubst $(src_dir)/%.c,$(obj_dir)/%.o,$(ulib_src))

CFLAGS += -nostdinc -static -no-pie -fno-builtin -ffreestanding -Wall
CFLAGS += -I$(inc_dir) -I$(ulib_dir)/../libax
LDFLAGS += -nostdlib -static -no-pie --gc-sections -T$(LD_SCRIPT)

ifeq ($(MODE), release)
  CFLAGS += -O3
endif

ifneq ($(wildcard $(in_feat)),)	# check if features.txt contains "fp_simd"
  fp_simd := $(shell grep "fp_simd" < $(in_feat))
endif

ifeq ($(ARCH), riscv64)
  CFLAGS += -march=rv64gc -mabi=lp64d -mcmodel=medany
endif

ifeq ($(fp_simd),)
  ifeq ($(ARCH), x86_64)
    CFLAGS += -mno-sse
  else ifeq ($(ARCH), aarch64)
    CFLAGS += -mgeneral-regs-only
  endif
endif

ifneq ($(wildcard $(in_feat)),)
_gen_feat: $(obj_dir)
  ifneq ($(shell diff -Nq $(in_feat) $(out_feat)),)
	$(shell cp $(in_feat) $(out_feat))
  endif
else
_gen_feat: $(obj_dir)
  ifneq ($(shell cat $(out_feat) 2> /dev/null),default)
	@echo default > $(out_feat)
  endif
endif

$(obj_dir):
	mkdir -p $@

$(obj_dir)/%.o: $(src_dir)/%.c $(out_feat)
	$(CC) $(CFLAGS) -c -o $@ $<

$(c_lib): $(obj_dir) _gen_feat $(ulib_obj)
	rm -f $@
	$(AR) rc $@ $(ulib_obj)
	$(RANLIB) $@

app-objs := main.o

-include $(APP)/axbuild.mk  # override `app-objs`

app-objs := $(addprefix $(APP)/,$(app-objs))

$(APP)/%.o: $(APP)/%.c
	$(CC) $(CFLAGS) -c -o $@ $<

$(OUT_ELF): $(app-objs) $(c_lib) $(rust_lib)
	@printf "    $(CYAN_C)Linking$(END_C) $(OUT_ELF)\n"
	$(LD) $(LDFLAGS) $^ -o $@

.PHONY: _gen_feat
