# 用于链接应用程序到内核中

    .section .data.app
    .global app0_start
    .global app0_end
    .align 4
app0_start:
    .incbin "./apps/helloworld/helloworld"
app0_end:

    .section .data.app
    .global app1_start
    .global app1_end
    .align 4
app1_start:
    .incbin "./apps/helloworld/second"
app1_end:
