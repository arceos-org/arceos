
    .section .data
    .global img_start
    .global img_end
    .align 16
img_start:
    .incbin "./sdcard.img"
img_end:
