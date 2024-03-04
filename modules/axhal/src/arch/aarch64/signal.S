# To create the sigreturn trampoline
.equ __NR_sigreturn, 139
.section .text.signal_trampoline
.balign 4
.global start_signal_trampoline
start_signal_trampoline: