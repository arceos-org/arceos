# To create the sigreturn trampoline

.section .text.signal_trampoline
.code64
.global start_signal_trampoline
start_signal_trampoline:
    # syscall id rdi = 15
    mov rax, 0xf
    syscall