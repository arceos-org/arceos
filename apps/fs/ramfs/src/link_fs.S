    .align 4
    .section .data
    .global fs_
fs_:
    .quad 2
    .quad fs_start

    .section .data
    .global fs_start
    .global fs_end
fs_start:
    .incbin "apps/fs/ramfs/fat32.img"
fs_end:

