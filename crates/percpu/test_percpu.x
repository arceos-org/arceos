CPU_NUM = 4;

SECTIONS
{
    percpu_start = .;
    .percpu 0x0 (NOLOAD) : AT(percpu_start) ALIGN(4K) {
        __percpu_offset_start = .;
        *(.percpu .percpu.*)
        __percpu_offset_end = .;
        . = ALIGN(4K);
        __percpu_size_aligned = .;

        . = __percpu_offset_start + __percpu_size_aligned * CPU_NUM;
    }
    . = percpu_start + SIZEOF(.percpu);
    percpu_end = .;
}
INSERT BEFORE .bss;
