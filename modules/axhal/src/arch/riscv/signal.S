.section .text.signal_trampoline
.balign 4
.global start_signal_trampoline
start_signal_trampoline:
    li a7, 139
    ecall