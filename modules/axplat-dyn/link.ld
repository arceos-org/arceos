/* 
 * Test linker script that works for both architectures
 * The actual OUTPUT_ARCH will be determined by the target
 */

INCLUDE "link.x"
__SMP = {{SMP}};
SECTIONS {
    linkme_IRQ : { KEEP(*(linkme_IRQ)) }
    linkm2_IRQ : { KEEP(*(linkm2_IRQ)) }
    linkme_PAGE_FAULT : { KEEP(*(linkme_PAGE_FAULT)) }
    linkm2_PAGE_FAULT : { KEEP(*(linkm2_PAGE_FAULT)) }
    scope_local : { KEEP(*(scope_local)) }
}
INSERT AFTER .tbss;

SECTIONS {
    debug_abbrev : { . += SIZEOF(.debug_abbrev); }
    debug_addr : { . += SIZEOF(.debug_addr); }
    debug_aranges : { . += SIZEOF(.debug_aranges); }
    debug_info : { . += SIZEOF(.debug_info); }
    debug_line : { . += SIZEOF(.debug_line); }
    debug_line_str : { . += SIZEOF(.debug_line_str); }
    debug_ranges : { . += SIZEOF(.debug_ranges); }
    debug_rnglists : { . += SIZEOF(.debug_rnglists); }
    debug_str : { . += SIZEOF(.debug_str); }
    debug_str_offsets : { . += SIZEOF(.debug_str_offsets); }
}
INSERT AFTER .rodata;

SECTIONS { 
    Ex_table : ALIGN(0x10){ 
        _ex_table_start = .;
        KEEP(*(__ex_table))
        _ex_table_end = .;
    }
}
INSERT AFTER .data;

_sbss = .;
_ebss = .;
boot_stack = .;
boot_stack_top = .;
_percpu_load_start = __percpu_start;

_skernel = VM_LOAD_ADDRESS;
_ekernel = __kernel_code_end;