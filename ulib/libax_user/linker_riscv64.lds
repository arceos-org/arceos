OUTPUT_ARCH(riscv)

BASE_ADDRESS = 0x04000000;

ENTRY(_start)
SECTIONS
{
    . = BASE_ADDRESS;

    .text : ALIGN(4K) {
        stext = .;
        *(.text.start)
        *(.text .text.*)
        . = ALIGN(4K);
        etext = .;
    }

    .rodata : ALIGN(4K) {
        srodata = .;
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        . = ALIGN(4K);
        erodata = .;
    }

    .data : ALIGN(4K) {
        sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        *(.got .got.*)
        . = ALIGN(4K);
        edata = .;
    }

    .bss : ALIGN(4K) {
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        . = ALIGN(4K);
        ebss = .;
    }
        
    /DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}
