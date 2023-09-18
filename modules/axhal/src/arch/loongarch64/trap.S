.comm    kernelsp, 0x1000, 0x1000

.macro SAVE_STATIC_REGS
	st.d	$s0, $sp, 8*23
	st.d	$s1, $sp, 8*24
	st.d	$s2, $sp, 8*25
	st.d	$s3, $sp, 8*26
	st.d	$s4, $sp, 8*27
	st.d	$s5, $sp, 8*28
	st.d	$s6, $sp, 8*29
	st.d	$s7, $sp, 8*30
	st.d	$s8, $sp, 8*31
.endm

.macro LOAD_STATIC_REGS
	ld.d	$s0, $sp, 8*23
	ld.d	$s1, $sp, 8*24
	ld.d	$s2, $sp, 8*25
	ld.d	$s3, $sp, 8*26
	ld.d	$s4, $sp, 8*27
	ld.d	$s5, $sp, 8*28
	ld.d	$s6, $sp, 8*29
	ld.d	$s7, $sp, 8*30
	ld.d	$s8, $sp, 8*31
.endm

.macro SAVE_ARGUMENT_REGS
	st.d	$a0, $sp, 8*4
	st.d	$a1, $sp, 8*5
	st.d	$a2, $sp, 8*6
	st.d	$a3, $sp, 8*7
	st.d	$a4, $sp, 8*8
	st.d	$a5, $sp, 8*9
	st.d	$a6, $sp, 8*10
	st.d	$a7, $sp, 8*11
.endm

.macro LOAD_ARGUMENT_REGS
	ld.d	$a0, $sp, 8*4
	ld.d	$a1, $sp, 8*5
	ld.d	$a2, $sp, 8*6
	ld.d	$a3, $sp, 8*7
	ld.d	$a4, $sp, 8*8
	ld.d	$a5, $sp, 8*9
	ld.d	$a6, $sp, 8*10
	ld.d	$a7, $sp, 8*11
.endm

.macro SAVE_TEMP_REGS
	st.d	$t0, $sp, 8*12
	st.d	$t1, $sp, 8*13
	st.d	$t2, $sp, 8*14
	st.d	$t3, $sp, 8*15
	st.d	$t4, $sp, 8*16
	st.d	$t5, $sp, 8*17
	st.d	$t6, $sp, 8*18
	st.d	$t7, $sp, 8*19
	st.d	$t8, $sp, 8*20
.endm

.macro LOAD_TEMP_REGS
	ld.d	$t0, $sp, 8*12
	ld.d	$t1, $sp, 8*13
	ld.d	$t2, $sp, 8*14
	ld.d	$t3, $sp, 8*15
	ld.d	$t4, $sp, 8*16
	ld.d	$t5, $sp, 8*17
	ld.d	$t6, $sp, 8*18
	ld.d	$t7, $sp, 8*19
	ld.d	$t8, $sp, 8*20
.endm

.macro SAVE_CSR_REGS
    csrrd		$t2, 0x1
	st.d		$t2, $sp, 8*33  // prmd
	csrrd		$t2, 0x0
	st.d		$t2, $sp, 8*32  // crmd
	csrrd		$t2, 0x2
	st.d		$t2, $sp, 8*34  // euen
	csrrd		$t2, 0x4
	st.d		$t2, $sp, 8*35  // ecfg
	csrrd		$t2, 0x5
	st.d		$t2, $sp, 8*36  // estat
.endm

.macro LOAD_SOME_REGS
	ld.d	$a0, $sp, 8*33  // prmd
	andi    $a0, $a0, 0x3   // extract pplv bit (previous privilege level)
	beqz    $a0, 8f
	ld.d    $r21, $sp, 8*21 // reserved reg
8:
	ld.d	$a0, $sp, 8*37  // era
    addi.d 	$a0, $a0, 0x4   // era + 4
	csrwr	$a0, 0x6
	ld.d	$a0, $sp, 8*33  // prmd
	csrwr	$a0, 0x1
	ld.d	$ra, $sp, 8*1   // ra

    LOAD_ARGUMENT_REGS

	ld.d	$tp, $sp, 8*2   // thread pointer
	ld.d	$fp, $sp, 8*22  // frame pointer
.endm


.macro SAVE_REGS
	addi.d		$sp, $sp, -{trapframe_size} // allocate space
	st.d		$sp, $sp, 8*3   // sp
    st.d		$ra, $sp, 8*1   // ra

    SAVE_ARGUMENT_REGS
    SAVE_TEMP_REGS
    SAVE_STATIC_REGS
    SAVE_CSR_REGS

	csrrd		$ra, 0x6
	st.d		$ra, $sp, 8*37  // era

	st.d		$tp, $sp, 8*2   // thread pointer
	st.d		$r21, $sp, 8*21 // reserved reg
	st.d		$fp, $sp, 8*22  // frame pointer

	move	$a0, $sp
.endm


.macro LOAD_REGS
	ld.d	$sp, $sp, 8*3
    LOAD_STATIC_REGS

    LOAD_TEMP_REGS

    LOAD_SOME_REGS

	//ld.d	$sp, $sp, 8*3   // sp
	addi.d		$sp, $sp, {trapframe_size}

.endm


.section .text
.balign 4096
.global trap_vector_base
trap_vector_base:
    SAVE_REGS

    bl loongarch64_trap_handler

    LOAD_REGS

    ertn


