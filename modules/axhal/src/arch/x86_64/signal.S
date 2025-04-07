.section .text.signal_trampoline
.code64
.global start_signal_trampoline
start_signal_trampoline:
    mov rax, 0xf
    syscall