.macro SAVE_REGS
    addi    sp, sp, -{trapframe_size}
    PUSH_GENERAL_REGS

    csrr    t0, sepc
    csrr    t1, sstatus
    csrrw   t2, sscratch, zero          // save sscratch (sp) and zero it
    STR     t0, sp, 31                  // tf.sepc
    STR     t1, sp, 32                  // tf.sstatus
    STR     t2, sp, 1                   // tf.regs.sp

.endm

.macro RESTORE_REGS
    LDR     t0, sp, 31
    LDR     t1, sp, 32
    csrw    sepc, t0
    csrw    sstatus, t1

    POP_GENERAL_REGS
    LDR     sp, sp, 1                   // load sp from tf.regs.sp
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
