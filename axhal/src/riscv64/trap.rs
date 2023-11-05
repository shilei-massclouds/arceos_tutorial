use riscv::register::stvec;
use riscv::register::scause::{self, Exception as E, Trap};
use super::context::TrapFrame;

core::arch::global_asm!(
    include_str!("trap.S"),
    trapframe_size = const core::mem::size_of::<TrapFrame>(),
);

/// Writes Supervisor Trap Vector Base Address Register (`stvec`).
#[inline]
pub fn set_trap_vector_base(stvec: usize) {
    unsafe { stvec::write(stvec, stvec::TrapMode::Direct) }
}

#[no_mangle]
fn riscv_trap_handler(tf: &mut TrapFrame) {
    let scause = scause::read();
    match scause.cause() {
        Trap::Exception(E::Breakpoint) => handle_breakpoint(&mut tf.sepc),
        _ => {
            panic!(
                "Unhandled trap {:?} @ {:#x}:\n{:#x?}",
                scause.cause(),
                tf.sepc,
                tf
            );
        }
    }
}

fn handle_breakpoint(sepc: &mut usize) {
    log::debug!("Exception(Breakpoint) @ {:#x} ", sepc);
    *sepc += 2
}
