.section .text.signal_trampoline
.balign 4
.global start_signal_trampoline
start_signal_trampoline:
    li.w    $a7, 139
    syscall 0