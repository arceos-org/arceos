#define INVTLB_ADDR_GFALSE_AND_ASID	5
#define PAGE_SHIFT          12
#define PTRS_PER_PGD_BITS	(PAGE_SHIFT - 3)
#define PTRS_PER_PUD_BITS	(PAGE_SHIFT - 3)
#define PTRS_PER_PMD_BITS	(PAGE_SHIFT - 3)
#define PTRS_PER_PTE_BITS	(PAGE_SHIFT - 3)

/* Page table bits */
#define	_PAGE_VALID_SHIFT	0
#define	_PAGE_ACCESSED_SHIFT	0  /* Reuse Valid for Accessed */
#define	_PAGE_DIRTY_SHIFT	1
#define	_PAGE_PLV_SHIFT		2  /* 2~3, two bits */
#define	_CACHE_SHIFT		4  /* 4~5, two bits */
#define	_PAGE_GLOBAL_SHIFT	6
#define	_PAGE_HUGE_SHIFT	6  /* HUGE is a PMD bit */
#define	_PAGE_PRESENT_SHIFT	7
#define	_PAGE_WRITE_SHIFT	8
#define	_PAGE_MODIFIED_SHIFT	9
#define	_PAGE_PROTNONE_SHIFT	10
#define	_PAGE_SPECIAL_SHIFT	11
#define	_PAGE_HGLOBAL_SHIFT	12 /* HGlobal is a PMD bit */
#define	_PAGE_PFN_SHIFT		12
#define	_PAGE_SWP_EXCLUSIVE_SHIFT 23
#define	_PAGE_PFN_END_SHIFT	48
#define	_PAGE_NO_READ_SHIFT	61
#define	_PAGE_NO_EXEC_SHIFT	62
#define	_PAGE_RPLV_SHIFT	63

/* Used by software */
#define _PAGE_PRESENT		(_ULCAST_(1) << _PAGE_PRESENT_SHIFT)
#define _PAGE_WRITE		(_ULCAST_(1) << _PAGE_WRITE_SHIFT)
#define _PAGE_ACCESSED		(_ULCAST_(1) << _PAGE_ACCESSED_SHIFT)
#define _PAGE_MODIFIED		(_ULCAST_(1) << _PAGE_MODIFIED_SHIFT)
#define _PAGE_PROTNONE		(_ULCAST_(1) << _PAGE_PROTNONE_SHIFT)
#define _PAGE_SPECIAL		(_ULCAST_(1) << _PAGE_SPECIAL_SHIFT)

/* We borrow bit 23 to store the exclusive marker in swap PTEs. */
#define _PAGE_SWP_EXCLUSIVE	(_ULCAST_(1) << _PAGE_SWP_EXCLUSIVE_SHIFT)

/* Used by TLB hardware (placed in EntryLo*) */
#define _PAGE_VALID		(_ULCAST_(1) << _PAGE_VALID_SHIFT)
#define _PAGE_DIRTY		(_ULCAST_(1) << _PAGE_DIRTY_SHIFT)
#define _PAGE_PLV		(_ULCAST_(3) << _PAGE_PLV_SHIFT)
#define _PAGE_GLOBAL		(_ULCAST_(1) << _PAGE_GLOBAL_SHIFT)
#define _PAGE_HUGE		(_ULCAST_(1) << _PAGE_HUGE_SHIFT)
#define _PAGE_HGLOBAL		(_ULCAST_(1) << _PAGE_HGLOBAL_SHIFT)
#define _PAGE_NO_READ		(_ULCAST_(1) << _PAGE_NO_READ_SHIFT)
#define _PAGE_NO_EXEC		(_ULCAST_(1) << _PAGE_NO_EXEC_SHIFT)
#define _PAGE_RPLV		(_ULCAST_(1) << _PAGE_RPLV_SHIFT)
#define _CACHE_MASK		(_ULCAST_(3) << _CACHE_SHIFT)
#define _PFN_SHIFT		(PAGE_SHIFT - 12 + _PAGE_PFN_SHIFT)

#define _PAGE_USER	(PLV_USER << _PAGE_PLV_SHIFT)
#define _PAGE_KERN	(PLV_KERN << _PAGE_PLV_SHIFT)

/*
.macro tlb_do_page_fault, write
	SAVE_REGS

	csrrd		$a2, 0x7
	move		$a0, $sp
	st.d		$a2, $sp, 8*38
	li.d		$a1, \write
	bl			do_page_fault

	LOAD_REGS

	ertn
.endm
*/

.macro LOAD_PTE
    csrrd $t0, 0x1B	// pgd
    //lddir $t0, $t0, 4
	//bstrpick.d $t0, $t0, 63, 12
	//slli.d $t0, $t0, 12
	lddir $t0, $t0, 3 
	bstrpick.d $t0, $t0, 63, 12
	slli.d $t0, $t0, 12
	lddir $t0, $t0, 2
	bstrpick.d $t0, $t0, 63, 12
	slli.d $t0, $t0, 12
    lddir $t0, $t0, 1 
	bstrpick.d $t0, $t0, 63, 12
	slli.d $t0, $t0, 12
    ldpte $t0, 0	#取回偶数号页表项
		csrrd $t1, 0x8c	// tlbrelo0, used to debug
    ldpte $t0, 1 	#取回奇数号页表项
		csrrd $t1, 0x8d // tlbrelo1, used to debug
.endm

.section .text.tlbrefill
.balign 4096
.global tlb_refill_handler
tlb_refill_handler:
    csrwr $t0, 0x8B	// tlbrsave
	csrwr $t1, 0x30	// save

	LOAD_PTE
	
    tlbfill
    csrrd $t0, 0x8B	// tlbrsave
	csrrd $t1, 0x30	// save

    ertn
