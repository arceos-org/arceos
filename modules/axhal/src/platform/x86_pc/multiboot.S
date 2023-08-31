# Bootstrapping from 32-bit with the Multiboot specification.
# See https://www.gnu.org/software/grub/manual/multiboot/multiboot.html

.section .text.boot
.code32
.global _start
_start:
    mov     edi, eax        # arg1: magic: 0x2BADB002
    mov     esi, ebx        # arg2: multiboot info
    jmp     bsp_entry32

.balign 4
.type multiboot_header, STT_OBJECT
multiboot_header:
    .int    {mb_hdr_magic}                      # magic: 0x1BADB002
    .int    {mb_hdr_flags}                      # flags
    .int    -({mb_hdr_magic} + {mb_hdr_flags})  # checksum
    .int    multiboot_header - {offset}         # header_addr
    .int    _skernel - {offset}                 # load_addr
    .int    _edata - {offset}                   # load_end
    .int    _ebss - {offset}                    # bss_end_addr
    .int    _start - {offset}                   # entry_addr

# Common code in 32-bit, prepare states to enter 64-bit.
.macro ENTRY32_COMMON
    # set data segment selectors
    mov     ax, 0x18
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax

    # set PAE, PGE bit in CR4
    mov     eax, {cr4}
    mov     cr4, eax

    # load the temporary page table
    lea     eax, [.Ltmp_pml4 - {offset}]
    mov     cr3, eax

    # set LME, NXE bit in IA32_EFER
    mov     ecx, {efer_msr}
    mov     edx, 0
    mov     eax, {efer}
    wrmsr

    # set protected mode, write protect, paging bit in CR0
    mov     eax, {cr0}
    mov     cr0, eax
.endm

# Common code in 64-bit
.macro ENTRY64_COMMON
    # clear segment selectors
    xor     ax, ax
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax
.endm

.code32
bsp_entry32:
    lgdt    [.Ltmp_gdt_desc - {offset}]             # load the temporary GDT
    ENTRY32_COMMON
    ljmp    0x10, offset bsp_entry64 - {offset}    # 0x10 is code64 segment

.code32
.global ap_entry32
ap_entry32:
    ENTRY32_COMMON
    ljmp    0x10, offset ap_entry64 - {offset}     # 0x10 is code64 segment

.code64
bsp_entry64:
    ENTRY64_COMMON

    # set RSP to boot stack
    movabs  rsp, offset {boot_stack}
    add     rsp, {boot_stack_size}

    # call rust_entry(magic, mbi)
    movabs  rax, offset {entry}
    call    rax
    jmp     .Lhlt

.code64
ap_entry64:
    ENTRY64_COMMON

    # set RSP to high address (already set in ap_start.S)
    mov     rax, {offset}
    add     rsp, rax

    # call rust_entry_secondary(magic)
    mov     rdi, {mb_magic}
    movabs  rax, offset {entry_secondary}
    call    rax
    jmp     .Lhlt

.Lhlt:
    hlt
    jmp     .Lhlt

.section .rodata
.balign 8
.Ltmp_gdt_desc:
    .short  .Ltmp_gdt_end - .Ltmp_gdt - 1   # limit
    .long   .Ltmp_gdt - {offset}            # base

.section .data
.balign 16
.Ltmp_gdt:
    .quad 0x0000000000000000    # 0x00: null
    .quad 0x00cf9b000000ffff    # 0x08: code segment (base=0, limit=0xfffff, type=32bit code exec/read, DPL=0, 4k)
    .quad 0x00af9b000000ffff    # 0x10: code segment (base=0, limit=0xfffff, type=64bit code exec/read, DPL=0, 4k)
    .quad 0x00cf93000000ffff    # 0x18: data segment (base=0, limit=0xfffff, type=32bit data read/write, DPL=0, 4k)
.Ltmp_gdt_end:

.balign 4096
.Ltmp_pml4:
    # 0x0000_0000 ~ 0xffff_ffff
    .quad .Ltmp_pdpt_low - {offset} + 0x3   # PRESENT | WRITABLE | paddr(tmp_pdpt)
    .zero 8 * 510
    # 0xffff_ff80_0000_0000 ~ 0xffff_ff80_ffff_ffff
    .quad .Ltmp_pdpt_high - {offset} + 0x3  # PRESENT | WRITABLE | paddr(tmp_pdpt)

# FIXME: may not work on macOS using hvf as the CPU does not support 1GB page (pdpe1gb)
.Ltmp_pdpt_low:
    .quad 0x0000 | 0x83         # PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .quad 0x40000000 | 0x83     # PRESENT | WRITABLE | HUGE_PAGE | paddr(0x4000_0000)
    .quad 0x80000000 | 0x83     # PRESENT | WRITABLE | HUGE_PAGE | paddr(0x8000_0000)
    .quad 0xc0000000 | 0x83     # PRESENT | WRITABLE | HUGE_PAGE | paddr(0xc000_0000)
    .zero 8 * 508

.Ltmp_pdpt_high:
    .quad 0x0000 | 0x83         # PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .quad 0x40000000 | 0x83     # PRESENT | WRITABLE | HUGE_PAGE | paddr(0x4000_0000)
    .quad 0x80000000 | 0x83     # PRESENT | WRITABLE | HUGE_PAGE | paddr(0x8000_0000)
    .quad 0xc0000000 | 0x83     # PRESENT | WRITABLE | HUGE_PAGE | paddr(0xc000_0000)
    .zero 8 * 508
