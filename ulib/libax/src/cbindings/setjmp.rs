use super::ctypes;

/// `setjmp` implementation
///
/// TODO: only aarch64 is supported currently
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
    #[cfg(not(target_arch = "aarch64"))]
    core::arch::asm!("ret", options(noreturn))
}
