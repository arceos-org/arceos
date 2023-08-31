use core::ffi::c_int;

use super::ctypes;

/// `setjmp` implementation
#[naked]
#[no_mangle]
unsafe extern "C" fn setjmp(_buf: *mut ctypes::__jmp_buf_tag) {
    #[cfg(all(target_arch = "aarch64", feature = "fp_simd"))]
    core::arch::asm!(
        "
        stp x19, x20, [x0,#0]
        stp x21, x22, [x0,#16]
        stp x23, x24, [x0,#32]
        stp x25, x26, [x0,#48]
        stp x27, x28, [x0,#64]
        stp x29, x30, [x0,#80]
        mov x2, sp
        str x2, [x0,#104]
        stp  d8,  d9, [x0,#112]
        stp d10, d11, [x0,#128]
        stp d12, d13, [x0,#144]
        stp d14, d15, [x0,#160]
        mov x0, #0
        ret",
        options(noreturn),
    );
    #[cfg(all(target_arch = "aarch64", not(feature = "fp_simd")))]
    core::arch::asm!(
        "
        stp x19, x20, [x0,#0]
        stp x21, x22, [x0,#16]
        stp x23, x24, [x0,#32]
        stp x25, x26, [x0,#48]
        stp x27, x28, [x0,#64]
        stp x29, x30, [x0,#80]
        mov x2, sp
        str x2, [x0,#104]
        mov x0, #0
        ret",
        options(noreturn),
    );
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!(
        "mov [rdi], rbx
        mov [rdi + 8], rbp
        mov [rdi + 16], r12
        mov [rdi + 24], r13
        mov [rdi + 32], r14
        mov [rdi + 40], r15
        lea rdx, [rsp + 8]
        mov [rdi + 48], rdx
        mov rdx, [rsp]
        mov [rdi + 56], rdx
        xor rax, rax
        ret",
        options(noreturn),
    );
    #[cfg(all(target_arch = "riscv64", feature = "fp_simd"))]
    core::arch::asm!(
        "sd s0,    0(a0)
        sd s1,    8(a0)
        sd s2,    16(a0)
        sd s3,    24(a0)
        sd s4,    32(a0)
        sd s5,    40(a0)
        sd s6,    48(a0)
        sd s7,    56(a0)
        sd s8,    64(a0)
        sd s9,    72(a0)
        sd s10,   80(a0)
        sd s11,   88(a0)
        sd sp,    96(a0)
        sd ra,    104(a0)

        fsd fs0,  112(a0)
        fsd fs1,  120(a0)
        fsd fs2,  128(a0)
        fsd fs3,  136(a0)
        fsd fs4,  144(a0)
        fsd fs5,  152(a0)
        fsd fs6,  160(a0)
        fsd fs7,  168(a0)
        fsd fs8,  176(a0)
        fsd fs9,  184(a0)
        fsd fs10, 192(a0)
        fsd fs11, 200(a0)

        li a0, 0
        ret",
        options(noreturn),
    );
    #[cfg(all(target_arch = "riscv64", not(feature = "fp_simd")))]
    core::arch::asm!(
        "sd s0,    0(a0)
        sd s1,    8(a0)
        sd s2,    16(a0)
        sd s3,    24(a0)
        sd s4,    32(a0)
        sd s5,    40(a0)
        sd s6,    48(a0)
        sd s7,    56(a0)
        sd s8,    64(a0)
        sd s9,    72(a0)
        sd s10,   80(a0)
        sd s11,   88(a0)
        sd sp,    96(a0)
        sd ra,    104(a0)

        li a0, 0
        ret",
        options(noreturn),
    );
    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "x86_64",
        target_arch = "riscv64"
    )))]
    core::arch::asm!("ret", options(noreturn))
}

/// `longjmp` implementation
#[naked]
#[no_mangle]
unsafe extern "C" fn longjmp(_buf: *mut ctypes::__jmp_buf_tag, _val: c_int) -> ! {
    #[cfg(all(target_arch = "aarch64", feature = "fp_simd"))]
    core::arch::asm!(
        "ldp x19, x20, [x0,#0]
        ldp x21, x22, [x0,#16]
        ldp x23, x24, [x0,#32]
        ldp x25, x26, [x0,#48]
        ldp x27, x28, [x0,#64]
        ldp x29, x30, [x0,#80]
        ldr x2, [x0,#104]
        mov sp, x2
        ldp d8 , d9, [x0,#112]
        ldp d10, d11, [x0,#128]
        ldp d12, d13, [x0,#144]
        ldp d14, d15, [x0,#160]

        cmp w1, 0
        csinc w0, w1, wzr, ne
        br x30",
        options(noreturn),
    );
    #[cfg(all(target_arch = "aarch64", not(feature = "fp_simd")))]
    core::arch::asm!(
        "ldp x19, x20, [x0,#0]
        ldp x21, x22, [x0,#16]
        ldp x23, x24, [x0,#32]
        ldp x25, x26, [x0,#48]
        ldp x27, x28, [x0,#64]
        ldp x29, x30, [x0,#80]
        ldr x2, [x0,#104]
        mov sp, x2

        cmp w1, 0
        csinc w0, w1, wzr, ne
        br x30",
        options(noreturn),
    );
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!(
        "mov rax,rsi
        test rax,rax
        jnz 1f
        inc rax
    1:
        mov rbx, [rdi]
        mov rbp, [rdi + 8]
        mov r12, [rdi + 16]
        mov r13, [rdi + 24]
        mov r14, [rdi + 32]
        mov r15, [rdi + 40]
        mov rdx, [rdi + 48]
        mov rsp, rdx
        mov rdx, [rdi + 56]
        jmp rdx",
        options(noreturn),
    );
    #[cfg(all(target_arch = "riscv64", feature = "fp_simd"))]
    core::arch::asm!(
        "ld s0,    0(a0)
        ld s1,    8(a0)
        ld s2,    16(a0)
        ld s3,    24(a0)
        ld s4,    32(a0)
        ld s5,    40(a0)
        ld s6,    48(a0)
        ld s7,    56(a0)
        ld s8,    64(a0)
        ld s9,    72(a0)
        ld s10,   80(a0)
        ld s11,   88(a0)
        ld sp,    96(a0)
        ld ra,    104(a0)

        fld fs0,  112(a0)
        fld fs1,  120(a0)
        fld fs2,  128(a0)
        fld fs3,  136(a0)
        fld fs4,  144(a0)
        fld fs5,  152(a0)
        fld fs6,  160(a0)
        fld fs7,  168(a0)
        fld fs8,  176(a0)
        fld fs9,  184(a0)
        fld fs10, 192(a0)
        fld fs11, 200(a0)

        seqz a0, a1
        add a0, a0, a1
        ret",
        options(noreturn),
    );
    #[cfg(all(target_arch = "riscv64", not(feature = "fp_simd")))]
    core::arch::asm!(
        "ld s0,    0(a0)
        ld s1,    8(a0)
        ld s2,    16(a0)
        ld s3,    24(a0)
        ld s4,    32(a0)
        ld s5,    40(a0)
        ld s6,    48(a0)
        ld s7,    56(a0)
        ld s8,    64(a0)
        ld s9,    72(a0)
        ld s10,   80(a0)
        ld s11,   88(a0)
        ld sp,    96(a0)
        ld ra,    104(a0)

        seqz a0, a1
        add a0, a0, a1
        ret",
        options(noreturn),
    );
}
