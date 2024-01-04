.section .text
.balign 4096
.global trap_vector_base
trap_vector_base:
    addi.d		$sp, $sp, -{trapframe_size} // allocate space
    // save the registers.
    st.d $ra, $sp, 8
    st.d $tp, $sp, 16
    st.d $sp, $sp, 24
    st.d $a0, $sp, 32
    st.d $a1, $sp, 40
    st.d $a2, $sp, 48
    st.d $a3, $sp, 56
    st.d $a4, $sp, 64
    st.d $a5, $sp, 72
    st.d $a6, $sp, 80
    st.d $a7, $sp, 88
    st.d $t0, $sp, 96
    st.d $t1, $sp, 104
    st.d $t2, $sp, 112
    st.d $t3, $sp, 120
    st.d $t4, $sp, 128
    st.d $t5, $sp, 136
    st.d $t6, $sp, 144
    st.d $t7, $sp, 152
    st.d $t8, $sp, 160
    st.d $r21, $sp,168
    st.d $fp, $sp, 176
    st.d $s0, $sp, 184
    st.d $s1, $sp, 192
    st.d $s2, $sp, 200
    st.d $s3, $sp, 208
    st.d $s4, $sp, 216
    st.d $s5, $sp, 224
    st.d $s6, $sp, 232
    st.d $s7, $sp, 240
    st.d $s8, $sp, 248

    csrrd		$t2, 0x1
    st.d		$t2, $sp, 8*32  // prmd
    csrrd       $t1, 0x6        // era
    st.d        $t1, $sp, 8*33  // era

    move $a0, $sp
    bl loongarch64_trap_handler


    // restore the registers.
    ld.d  $t1, $sp, 8*33  // era
    csrwr $t1, 0x6
    ld.d  $t2, $sp, 8*32  // prmd
    csrwr $t2, 0x1

    ld.d $ra, $sp, 8
    ld.d $tp, $sp, 16
    ld.d $sp, $sp, 24
    ld.d $a0, $sp, 32
    ld.d $a1, $sp, 40
    ld.d $a2, $sp, 48
    ld.d $a3, $sp, 56
    ld.d $a4, $sp, 64
    ld.d $a5, $sp, 72
    ld.d $a6, $sp, 80
    ld.d $a7, $sp, 88
    ld.d $t0, $sp, 96
    ld.d $t1, $sp, 104
    ld.d $t2, $sp, 112
    ld.d $t3, $sp, 120
    ld.d $t4, $sp, 128
    ld.d $t5, $sp, 136
    ld.d $t6, $sp, 144
    ld.d $t7, $sp, 152
    ld.d $t8, $sp, 160
    ld.d $r21, $sp,168
    ld.d $fp, $sp, 176
    ld.d $s0, $sp, 184
    ld.d $s1, $sp, 192
    ld.d $s2, $sp, 200
    ld.d $s3, $sp, 208
    ld.d $s4, $sp, 216
    ld.d $s5, $sp, 224
    ld.d $s6, $sp, 232
    ld.d $s7, $sp, 240
    ld.d $s8, $sp, 248

    addi.d $sp, $sp, {trapframe_size}
    ertn


