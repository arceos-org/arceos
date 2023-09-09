
    .section .data
    .global img_start
    .global img_end
    .align 16
img_start:
    .incbin "./disk.img"
img_end:
