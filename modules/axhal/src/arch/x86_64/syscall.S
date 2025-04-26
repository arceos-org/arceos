.section .text
.code64
syscall_entry:
    swapgs                                                      // switch to kernel gs
    mov     gs:[offset __PERCPU_USER_RSP_OFFSET], rsp           // save user rsp
    mov     rsp, gs:[offset __PERCPU_TSS + {tss_rsp0_offset}]   // switch to kernel stack

    sub     rsp, 8                                  // skip user ss
    push    gs:[offset __PERCPU_USER_RSP_OFFSET]    // user rsp
    push    r11                                     // rflags
    push    {ucode64}                               // cs
    push    rcx                                     // rip
    sub     rsp, 4 * 8                              // skip until general registers

    push    r15
    push    r14
    push    r13
    push    r12
    push    r11
    push    r10
    push    r9
    push    r8
    push    rdi
    push    rsi
    push    rbp
    push    rbx
    push    rdx
    push    rcx
    push    rax

    mov     rdi, rsp
    call    x86_syscall_handler

    pop     rax
    pop     rcx
    pop     rdx
    pop     rbx
    pop     rbp
    pop     rsi
    pop     rdi
    pop     r8
    pop     r9
    pop     r10
    pop     r11
    pop     r12
    pop     r13
    pop     r14
    pop     r15

    add     rsp, 9 * 8
    mov     rcx, [rsp - 5 * 8]  // rip
    mov     r11, [rsp - 3 * 8]  // rflags
    mov     rsp, [rsp - 2 * 8]  // user rsp

    swapgs
    sysretq
