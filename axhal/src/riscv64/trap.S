.macro PUSH_POP_GENERAL_REGS, op
    \op ra, 0*8(sp)
    \op t0, 4*8(sp)
    \op t1, 5*8(sp)
    \op t2, 6*8(sp)
    \op s0, 7*8(sp)
    \op s1, 8*8(sp)
    \op a0, 9*8(sp)
    \op a1, 10*8(sp)
    \op a2, 11*8(sp)
    \op a3, 12*8(sp)
    \op a4, 13*8(sp)
    \op a5, 14*8(sp)
    \op a6, 15*8(sp)
    \op a7, 16*8(sp)
    \op s2, 17*8(sp)
    \op s3, 18*8(sp)
    \op s4, 19*8(sp)
    \op s5, 20*8(sp)
    \op s6, 21*8(sp)
    \op s7, 22*8(sp)
    \op s8, 23*8(sp)
    \op s9, 24*8(sp)
    \op s10, 25*8(sp)
    \op s11, 26*8(sp)
    \op t3, 27*8(sp)
    \op t4, 28*8(sp)
    \op t5, 29*8(sp)
    \op t6, 30*8(sp)
.endm

.macro PUSH_GENERAL_REGS
    PUSH_POP_GENERAL_REGS sd
.endm
.macro POP_GENERAL_REGS
    PUSH_POP_GENERAL_REGS ld
.endm

.macro SAVE_REGS
    addi    sp, sp, -{trapframe_size}
    PUSH_GENERAL_REGS

    csrr    t0, sepc
    csrr    t1, sstatus
    csrrw   t2, sscratch, zero          // save sscratch (sp) and zero it
    sd      t0, 31*8(sp)                // tf.sepc
    sd      t1, 32*8(sp)                // tf.sstatus
    sd      t2, 1*8(sp)                 // tf.regs.sp

.endm

.macro RESTORE_REGS
    ld     t0, 31*8(sp)
    ld     t1, 32*8(sp)
    csrw    sepc, t0
    csrw    sstatus, t1

    POP_GENERAL_REGS
    ld     sp, 1*8(sp)                  // load sp from tf.regs.sp
.endm

.section .text
.balign 4
.global trap_vector_base
trap_vector_base:
    csrrw   sp, sscratch, sp            // switch sscratch and sp
    csrr    sp, sscratch                // put supervisor sp back
    SAVE_REGS
    mv      a0, sp
    call    riscv_trap_handler
    RESTORE_REGS
    sret
