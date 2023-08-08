CPU_NUM = 4;

SECTIONS
{
    . = ALIGN(4K);
    _percpu_start = .;
    .percpu 0x0 (NOLOAD) : AT(_percpu_start) {
        _percpu_load_start = .;
        *(.percpu .percpu.*)
        _percpu_load_end = .;
        . = ALIGN(64);
        _percpu_size_aligned = .;

        . = _percpu_load_start + _percpu_size_aligned * CPU_NUM;
    }
    . = _percpu_start + SIZEOF(.percpu);
    _percpu_end = .;
}
INSERT BEFORE .bss;
