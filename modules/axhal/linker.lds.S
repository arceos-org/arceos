OUTPUT_ARCH(%ARCH%)

BASE_ADDRESS = %KERNEL_BASE%;

ENTRY(_start)
SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    .text : ALIGN(4K) {
        stext = .;
        *(.text.boot)
        *(.text .text.*)
        . = ALIGN(4K);
        etext = .;
    }

    .rodata : ALIGN(4K) {
        srodata = .;
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        *(.sdata2 .sdata2.*)
        . = ALIGN(4K);
        erodata = .;
    }

    .data : ALIGN(4K) {
        sdata = .;
        *(.data.boot_page_table)
        . = ALIGN(4K);
        *(.data .data.*)
        *(.sdata .sdata.*)
        *(.got .got.*)
        . = ALIGN(4K);
        edata = .;
    }

    percpu_start = .;
    .percpu 0x0 : AT(percpu_start) ALIGN(4K) {
        __percpu_offset_start = .;
        *(.percpu .percpu.*)
        __percpu_offset_end = .;
        . = ALIGN(4K);
        __percpu_size_aligned = .;

        . = __percpu_offset_start + __percpu_size_aligned * %SMP%;
    }
    . = percpu_start + SIZEOF(.percpu);
    percpu_end = .;

    .bss : ALIGN(4K) {
        boot_stack = .;
        *(.bss.stack)
        . = ALIGN(4K);
        boot_stack_top = .;

        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4K);
        ebss = .;
    }

    ekernel = .;

	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}
