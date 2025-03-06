use core::ffi::c_int;

use crate::ctypes;

/// `setjmp` implementation
#[naked]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setjmp(_buf: *mut ctypes::__jmp_buf_tag) {
    #[cfg(all(target_arch = "aarch64", feature = "fp_simd"))]
    core::arch::naked_asm!(
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
    );
    #[cfg(all(target_arch = "aarch64", not(feature = "fp_simd")))]
    core::arch::naked_asm!(
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
    );
    #[cfg(target_arch = "x86_64")]
    core::arch::naked_asm!(
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
    );
    #[cfg(all(target_arch = "riscv64", feature = "fp_simd"))]
    core::arch::naked_asm!(
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
    );
    #[cfg(all(target_arch = "riscv64", not(feature = "fp_simd")))]
    core::arch::naked_asm!(
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
    );
    #[cfg(all(target_arch = "loongarch64", feature = "fp_simd"))]
    core::arch::naked_asm!(
        "
        st.d     $ra, $a0, 0
        st.d     $sp, $a0, 1 * 8
        st.d     $s0, $a0, 2 * 8
        st.d     $s1, $a0, 3 * 8
        st.d     $s2, $a0, 4 * 8
        st.d     $s3, $a0, 5 * 8
        st.d     $s4, $a0, 6 * 8
        st.d     $s5, $a0, 7 * 8
        st.d     $s6, $a0, 8 * 8
        st.d     $s7, $a0, 9 * 8
        st.d     $s8, $a0, 10 * 8
        st.d     $fp, $a0, 11 * 8
        st.d     $r1, $a0, 12 * 8
        fst.d    $f24, $a0, 13 * 8
        fst.d    $f25, $a0, 14 * 8
        fst.d    $f26, $a0, 15 * 8
        fst.d    $f27, $a0, 16 * 8
        fst.d    $f28, $a0, 17 * 8
        fst.d    $f29, $a0, 18 * 8
        fst.d    $f30, $a0, 19 * 8
        fst.d    $f31, $a0, 20 * 8
        li.w  $a0, 0
        ret",
    );
    #[cfg(all(target_arch = "loongarch64", not(feature = "fp_simd")))]
    core::arch::naked_asm!(
        "
        st.d     $ra, $a0, 0
        st.d     $sp, $a0, 1 * 8
        st.d     $s0, $a0, 2 * 8
        st.d     $s1, $a0, 3 * 8
        st.d     $s2, $a0, 4 * 8
        st.d     $s3, $a0, 5 * 8
        st.d     $s4, $a0, 6 * 8
        st.d     $s5, $a0, 7 * 8
        st.d     $s6, $a0, 8 * 8
        st.d     $s7, $a0, 9 * 8
        st.d     $s8, $a0, 10 * 8
        st.d     $fp, $a0, 11 * 8
        st.d     $r1, $a0, 12 * 8
        li.w  $a0, 0
        ret",
    );
    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "x86_64",
        target_arch = "riscv64",
        target_arch = "loongarch64"
    )))]
    core::arch::naked_asm!("ret")
}

/// `longjmp` implementation
#[naked]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn longjmp(_buf: *mut ctypes::__jmp_buf_tag, _val: c_int) -> ! {
    #[cfg(all(target_arch = "aarch64", feature = "fp_simd"))]
    core::arch::naked_asm!(
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
    );
    #[cfg(all(target_arch = "aarch64", not(feature = "fp_simd")))]
    core::arch::naked_asm!(
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
    );
    #[cfg(target_arch = "x86_64")]
    core::arch::naked_asm!(
        "mov rax,rsi
        test rax,rax
        jnz 2f
        inc rax
    2:
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
    );
    #[cfg(all(target_arch = "riscv64", feature = "fp_simd"))]
    core::arch::naked_asm!(
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
    );
    #[cfg(all(target_arch = "riscv64", not(feature = "fp_simd")))]
    core::arch::naked_asm!(
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
    );

    #[cfg(all(target_arch = "loongarch64", feature = "fp_simd"))]
    core::arch::naked_asm!(
        "
        ld.d     $ra, $a1, 0
        ld.d     $s0, $a1, 2 * 8
        ld.d     $s1, $a1, 3 * 8
        ld.d     $s2, $a1, 4 * 8
        ld.d     $s3, $a1, 5 * 8
        ld.d     $s4, $a1, 6 * 8
        ld.d     $s5, $a1, 7 * 8
        ld.d     $s6, $a1, 8 * 8
        ld.d     $s7, $a1, 9 * 8
        ld.d     $s8, $a1, 10 * 8
        ld.d     $fp, $a1, 11 * 8
        ld.d     $sp, $a1, 1 * 8
        ld.d     $r21, $a1, 12 * 8
        fld.d    $f24, $a0, 13 * 8
        fld.d    $f25, $a0, 14 * 8
        fld.d    $f26, $a0, 15 * 8
        fld.d    $f27, $a0, 16 * 8
        fld.d    $f28, $a0, 17 * 8
        fld.d    $f29, $a0, 18 * 8
        fld.d    $f30, $a0, 19 * 8
        fld.d    $f31, $a0, 20 * 8
        sltui    $a0, $a1, 1
        add.d    $a0, $a0, $a1
        jirl     $zero,$ra, 0"
    );
    #[cfg(all(target_arch = "loongarch64", not(feature = "fp_simd")))]
    core::arch::naked_asm!(
        "
        ld.d     $ra, $a1, 0
        ld.d     $s0, $a1, 2 * 8
        ld.d     $s1, $a1, 3 * 8
        ld.d     $s2, $a1, 4 * 8
        ld.d     $s3, $a1, 5 * 8
        ld.d     $s4, $a1, 6 * 8
        ld.d     $s5, $a1, 7 * 8
        ld.d     $s6, $a1, 8 * 8
        ld.d     $s7, $a1, 9 * 8
        ld.d     $s8, $a1, 10 * 8
        ld.d     $fp, $a1, 11 * 8
        ld.d     $sp, $a1, 1 * 8
        ld.d     $r21, $a1, 12 * 8
        sltui    $a0, $a1, 1
        add.d    $a0, $a0, $a1
        jirl     $zero,$ra, 0",
    );
}
