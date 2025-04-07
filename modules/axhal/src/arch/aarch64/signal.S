.section .text.signal_trampoline
.balign 4
.global start_signal_trampoline
start_signal_trampoline:
    mov x8, #139
    svc #0