# Boot application processors into the protected mode.

# Each non-boot CPU ("AP") is started up in response to a STARTUP
# IPI from the boot CPU.  Section B.4.2 of the Multi-Processor
# Specification says that the AP will start in real mode with CS:IP
# set to XY00:0000, where XY is an 8-bit value sent with the
# STARTUP. Thus this code must start at a 4096-byte boundary.
#
# Because this code sets DS to zero, it must sit
# at an address in the low 2^16 bytes.

.equ pa_ap_start32, ap_start32 - ap_start + {start_page_paddr}
.equ pa_ap_gdt, .Lap_tmp_gdt - ap_start + {start_page_paddr}
.equ pa_ap_gdt_desc, .Lap_tmp_gdt_desc - ap_start + {start_page_paddr}

.equ stack_ptr, {start_page_paddr} + 0xff0
.equ entry_ptr, {start_page_paddr} + 0xff8

# 0x6000
.section .text
.code16
.p2align 12
.global ap_start
ap_start:
    cli
    wbinvd

    xor     ax, ax
    mov     ds, ax
    mov     es, ax
    mov     ss, ax
    mov     fs, ax
    mov     gs, ax

    # load the 64-bit GDT
    lgdt    [pa_ap_gdt_desc]

    # switch to protected-mode
    mov     eax, cr0
    or      eax, (1 << 0)
    mov     cr0, eax

    # far jump to 32-bit code. 0x8 is code32 segment selector
    ljmp    0x8, offset pa_ap_start32

.code32
ap_start32:
    mov     esp, [stack_ptr]
    mov     eax, [entry_ptr]
    jmp     eax

.balign 8
# .type multiboot_header, STT_OBJECT
.Lap_tmp_gdt_desc:
    .short .Lap_tmp_gdt_end - .Lap_tmp_gdt - 1  # limit
    .long pa_ap_gdt                             # base

.balign 16
.Lap_tmp_gdt:
    .quad 0x0000000000000000    # 0x00: null
    .quad 0x00cf9b000000ffff    # 0x08: code segment (base=0, limit=0xfffff, type=32bit code exec/read, DPL=0, 4k)
    .quad 0x00af9b000000ffff    # 0x10: code segment (base=0, limit=0xfffff, type=64bit code exec/read, DPL=0, 4k)
    .quad 0x00cf93000000ffff    # 0x18: data segment (base=0, limit=0xfffff, type=32bit data read/write, DPL=0, 4k)
.Lap_tmp_gdt_end:

# 0x7000
.p2align 12
.global ap_end
ap_end:
