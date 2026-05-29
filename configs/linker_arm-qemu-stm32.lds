OUTPUT_ARCH(arm)

/* STM32F100RB: 128KB Flash at 0x08000000, 8KB SRAM at 0x20000000 */
FLASH_BASE = 0x08000000;
FLASH_SIZE = 0x20000;       /* 128KB */
SRAM_BASE  = 0x20000000;
SRAM_SIZE  = 0x2000;        /* 8KB */

ENTRY(_start)

MEMORY
{
    FLASH (RX)  : ORIGIN = FLASH_BASE, LENGTH = FLASH_SIZE
    SRAM (RW)   : ORIGIN = SRAM_BASE,  LENGTH = SRAM_SIZE
}

SECTIONS
{
    . = ORIGIN(FLASH);
    _skernel = .;

    /* Vector table MUST be first — Cortex-M reads initial SP and reset vector from here */
    .vector_table : ALIGN(4) {
        KEEP(*(.vector_table))
    } > FLASH

    .text : ALIGN(4) {
        _stext = .;
        *(.text.boot)
        *(.text .text.*)
        . = ALIGN(4);
        _etext = .;
    } > FLASH

    _srodata = .;
    .rodata : ALIGN(4) {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        *(.sdata2 .sdata2.*)
    } > FLASH

    .init_array : ALIGN(4) {
        __init_array_start = .;
        *(.init_array .init_array.*)
        __init_array_end = .;
    } > FLASH

    .ARM.exidx : {
        *(.ARM.exidx .ARM.exidx.*)
    } > FLASH

    . = ALIGN(4);
    _erodata = .;

    /* .data goes in SRAM but loaded from Flash (AT> FLASH) */
    . = ORIGIN(SRAM);
    _sidata = LOADADDR(.data);
    .data : AT(_erodata) ALIGN(4) {
        _sdata = .;
        KEEP(*(.data.boot_page_table))
        KEEP(*(.data .data.*))
        KEEP(*(.sdata .sdata.*))
        KEEP(*(.got .got.*))
        KEEP(*(linkme_ANY))
        KEEP(*(linkm2_ANY))
        KEEP(*(linkme_IRQ))
        KEEP(*(linkm2_IRQ))
        KEEP(*(linkme_PAGE_FAULT))
        KEEP(*(linkm2_PAGE_FAULT))
        KEEP(*(linkme_SYSCALL))
        KEEP(*(linkm2_SYSCALL))
        KEEP(*(axns_resource))

        . = ALIGN(64);
        _percpu_start = .;
        _percpu_load_start = .;
        KEEP(*(.percpu .percpu.*))
        _percpu_load_end = .;
        _percpu_end = .;
    } > SRAM

    .tdata : ALIGN(4) {
        _stdata = .;
        *(.tdata .tdata.*)
        _etdata = .;
    } > SRAM AT> FLASH

    .tbss : ALIGN(4) {
        _stbss = .;
        *(.tbss .tbss.*)
        *(.tcommon)
        _etbss = .;
    } > SRAM

    . = ALIGN(4);
    _edata = .;

    .bss.stack (NOLOAD) : ALIGN(8) {
        boot_stack = .;
        *(.bss.stack .bss.stack.*)
        . = ALIGN(8);
        boot_stack_top = .;
    } > SRAM

    /* BSS in SRAM (before stack, so BSS clearing won't corrupt stack frames) */
    .bss : ALIGN(4) {
        _sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4);
        _ebss = .;
    } > SRAM

    _ekernel = .;

    /DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}
