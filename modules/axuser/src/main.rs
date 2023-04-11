#![no_std]
#![no_main]

extern crate axruntime;

core::arch::global_asm!(r#"
    .section .rodata
    .globl ustart
    ustart:
    .incbin "./modules/axuser/user.bin"
    .globl uend
    uend:
"#
);
