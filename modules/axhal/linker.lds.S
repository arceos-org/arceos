OUTPUT_ARCH(%ARCH%)

BASE_ADDRESS = %KERNEL_BASE%;

ENTRY(_start)
SECTIONS
{
    . = BASE_ADDRESS;
    _skernel = .;

    .text : ALIGN(4K) {
        _stext = .;
        *(.text.boot)
        *(.text .text.*)
        . = ALIGN(4K);
        _etext = .;
    }

    _srodata = .;
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        *(.sdata2 .sdata2.*)
    }

    .init_array : ALIGN(0x10) {
        __init_array_start = .;
        *(.init_array .init_array.*)
        __init_array_end = .;
    }

    . = ALIGN(4K);
    _erodata = .;

    .data : ALIGN(4K) {
        _sdata = .;
        *(.data.boot_page_table)
        . = ALIGN(4K);
        *(.data .data.*)
        *(.sdata .sdata.*)
        *(.got .got.*)
    }

    .tdata : ALIGN(0x10) {
        _stdata = .;
        *(.tdata .tdata.*)
        _etdata = .;
    }

    .tbss : ALIGN(0x10) {
        _stbss = .;
        *(.tbss .tbss.*)
        *(.tcommon)
        _etbss = .;
    }

    . = ALIGN(4K);
    _percpu_start = .;
    _percpu_end = _percpu_start + SIZEOF(.percpu);
    .percpu 0x0 : AT(_percpu_start) {
        _percpu_load_start = .;
        *(.percpu .percpu.*)
        _percpu_load_end = .;
        . = _percpu_load_start + ALIGN(64) * %CPU_NUM%;
    }
    . = _percpu_end;

    . = ALIGN(4K);
    _edata = .;

    .bss : AT(.) ALIGN(4K) {
        boot_stack = .;
        *(.bss.stack)
        . = ALIGN(4K);
        boot_stack_top = .;

        _sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4K);
        _ebss = .;
    }

    _ekernel = .;

	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}

SECTIONS {
    linkme_IRQ : { *(linkme_IRQ) }
    linkm2_IRQ : { *(linkm2_IRQ) }
    linkme_PAGE_FAULT : { *(linkme_PAGE_FAULT) }
    linkm2_PAGE_FAULT : { *(linkm2_PAGE_FAULT) }
    linkme_SYSCALL : { *(linkme_SYSCALL) }
    linkm2_SYSCALL : { *(linkm2_SYSCALL) }
    axns_resource : { *(axns_resource) }
    
    .axtest : ALIGN(8) {
        __axtest_suites_start = .;
        KEEP(*(.axtest))
        __axtest_suites_end = .;
    }

    /* LLVM profiling sections */
    __llvm_prf_data : ALIGN(8) {
        PROVIDE(__start___llvm_prf_data = .);
        __llvm_prf_data_start = .;
        KEEP(*(__llvm_prf_data))
        PROVIDE(__stop___llvm_prf_data = .);
        __llvm_prf_data_end = .;
    }
    
    __llvm_prf_vnds : ALIGN(8) {
        PROVIDE(__start___llvm_prf_vnds = .);
        __llvm_prf_vnds_start = .;
        KEEP(*(__llvm_prf_vnds))
        PROVIDE(__stop___llvm_prf_vnds = .);
        __llvm_prf_vnds_end = .;
    }

    __llvm_prf_vns : ALIGN(8) {
        PROVIDE(__start___llvm_prf_vns = .);
        KEEP(*(__llvm_prf_vns))
        PROVIDE(__stop___llvm_prf_vns = .);
    }

    __llvm_prf_vtab : ALIGN(8) {
        PROVIDE(__start___llvm_prf_vtab = .);
        KEEP(*(__llvm_prf_vtab))
        PROVIDE(__stop___llvm_prf_vtab = .);
    }
    
    __llvm_prf_names : ALIGN(8) {
        PROVIDE(__start___llvm_prf_names = .);
        __llvm_prf_names_start = .;
        KEEP(*(__llvm_prf_names))
        PROVIDE(__stop___llvm_prf_names = .);
        __llvm_prf_names_end = .;
    }

    __llvm_prf_bits : ALIGN(8) {
        PROVIDE(__start___llvm_prf_bits = .);
        KEEP(*(__llvm_prf_bits))
        PROVIDE(__stop___llvm_prf_bits = .);
    }
    
    __llvm_prf_cnts : ALIGN(8) {
        PROVIDE(__start___llvm_prf_cnts = .);
        __llvm_prf_cnts_start = .;
        KEEP(*(__llvm_prf_cnts))
        PROVIDE(__stop___llvm_prf_cnts = .);
        __llvm_prf_cnts_end = .;
    }

    __llvm_orderfile : ALIGN(8) {
        PROVIDE(__start___llvm_orderfile = .);
        KEEP(*(__llvm_orderfile))
        PROVIDE(__stop___llvm_orderfile = .);
    }

}
INSERT AFTER .tbss;
