# To create the sigreturn trampoline
.equ __NR_sigreturn, 139
.section .text.signal_trampoline
.balign 4
.global start_signal_trampoline
start_signal_trampoline:
    mov x8, #139  // 设置系统调用号为 139
    svc #0        // 触发系统调用