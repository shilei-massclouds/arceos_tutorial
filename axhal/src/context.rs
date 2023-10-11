use core::arch::asm;
use memory_addr::VirtAddr;

macro_rules! include_asm_marcos {
    () => {
        core::arch::global_asm!(
            r"
        .ifndef XLENB
        .equ XLENB, 8

        .macro LDR rd, rs, off
            ld \rd, \off*XLENB(\rs)
        .endm
        .macro STR rs2, rs1, off
            sd \rs2, \off*XLENB(\rs1)
        .endm

        .endif",
        );

        core::arch::global_asm!(
            r"
        .ifndef .LPUSH_POP_GENERAL_REGS
        .equ .LPUSH_POP_GENERAL_REGS, 0

        .macro PUSH_POP_GENERAL_REGS, op
            \op ra, sp, 0
            \op t0, sp, 4
            \op t1, sp, 5
            \op t2, sp, 6
            \op s0, sp, 7
            \op s1, sp, 8
            \op a0, sp, 9
            \op a1, sp, 10
            \op a2, sp, 11
            \op a3, sp, 12
            \op a4, sp, 13
            \op a5, sp, 14
            \op a6, sp, 15
            \op a7, sp, 16
            \op s2, sp, 17
            \op s3, sp, 18
            \op s4, sp, 19
            \op s5, sp, 20
            \op s6, sp, 21
            \op s7, sp, 22
            \op s8, sp, 23
            \op s9, sp, 24
            \op s10, sp, 25
            \op s11, sp, 26
            \op t3, sp, 27
            \op t4, sp, 28
            \op t5, sp, 29
            \op t6, sp, 30
        .endm

        .macro PUSH_GENERAL_REGS
            PUSH_POP_GENERAL_REGS STR
        .endm
        .macro POP_GENERAL_REGS
            PUSH_POP_GENERAL_REGS LDR
        .endm

        .endif"
        );
    };
}

include_asm_marcos!();

/// General registers of RISC-V.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct GeneralRegisters {
    pub ra: usize,
    pub sp: usize,
    pub gp: usize, // only valid for user traps
    pub tp: usize, // only valid for user traps
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
}

/// Saved registers when a trap (interrupt or exception) occurs.
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct TrapFrame {
    /// All general registers.
    pub regs: GeneralRegisters,
    /// Supervisor Exception Program Counter.
    pub sepc: usize,
    /// Supervisor Status Register.
    pub sstatus: usize,
}

/// Saved hardware states of a task.
///
/// The context usually includes:
///
/// - Callee-saved registers
/// - Stack pointer register
/// - Thread pointer register (for thread-local storage, currently unsupported)
/// - FP/SIMD registers
///
/// On context switch, current task saves its context from CPU to memory,
/// and the next task restores its context from memory to CPU.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default)]
pub struct TaskContext {
    pub ra: usize, // return address (x1)
    pub sp: usize, // stack pointer (x2)

    pub s0: usize, // x8-x9
    pub s1: usize,

    pub s2: usize, // x18-x27
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,

    pub tp: usize,
    // TODO: FP states
}

impl TaskContext {
    /// Creates a new default context for a new task.
    pub const fn new() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    /// Initializes the context for a new task, with the given entry point and
    /// kernel stack.
    pub fn init(&mut self, entry: usize, kstack_top: VirtAddr, tls_area: VirtAddr) {
        self.sp = kstack_top.as_usize();
        self.ra = entry;
        self.tp = tls_area.as_usize();
    }

    /// Switches to another task.
    ///
    /// It first saves the current task's context from CPU to this place, and then
    /// restores the next task's context from `next_ctx` to CPU.
    pub fn switch_to(&mut self, next_ctx: &Self) {
        #[cfg(feature = "tls")]
        {
            self.tp = super::read_thread_pointer();
            unsafe { super::write_thread_pointer(next_ctx.tp) };
        }
        unsafe {
            // TODO: switch FP states
            context_switch(self, next_ctx)
        }
    }
}

#[naked]
unsafe extern "C" fn context_switch(_current_task: &mut TaskContext, _next_task: &TaskContext) {
    asm!(
        "
        // save old context (callee-saved registers)
        STR     ra, a0, 0
        STR     sp, a0, 1
        STR     s0, a0, 2
        STR     s1, a0, 3
        STR     s2, a0, 4
        STR     s3, a0, 5
        STR     s4, a0, 6
        STR     s5, a0, 7
        STR     s6, a0, 8
        STR     s7, a0, 9
        STR     s8, a0, 10
        STR     s9, a0, 11
        STR     s10, a0, 12
        STR     s11, a0, 13

        // restore new context
        LDR     s11, a1, 13
        LDR     s10, a1, 12
        LDR     s9, a1, 11
        LDR     s8, a1, 10
        LDR     s7, a1, 9
        LDR     s6, a1, 8
        LDR     s5, a1, 7
        LDR     s4, a1, 6
        LDR     s3, a1, 5
        LDR     s2, a1, 4
        LDR     s1, a1, 3
        LDR     s0, a1, 2
        LDR     sp, a1, 1
        LDR     ra, a1, 0

        ret",
        options(noreturn),
    )
}
