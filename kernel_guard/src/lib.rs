#![no_std]
#![feature(asm_const)]

use core::arch::asm;

/// Bit 1: Supervisor Interrupt Enable
const SIE_BIT: usize = 1 << 1;

/// A base trait that all guards implement.
pub trait BaseGuard {
    /// The saved state when entering the critical section.
    type State: Clone + Copy;

    /// Something that must be done before entering the critical section.
    fn acquire() -> Self::State;

    /// Something that must be done after leaving the critical section.
    fn release(state: Self::State);
}

pub struct NoPreemptIrqSave(usize);

impl BaseGuard for NoPreemptIrqSave {
    type State = usize;
    fn acquire() -> Self::State {
        // disable IRQs and save IRQ states
        local_irq_save_and_disable()
    }
    fn release(state: Self::State) {
        // restore IRQ states
        local_irq_restore(state);
    }
}

#[inline]
fn local_irq_save_and_disable() -> usize {
    let flags: usize;
    // clear the `SIE` bit, and return the old CSR
    unsafe { asm!("csrrc {}, sstatus, {}", out(reg) flags, const SIE_BIT) };
    flags & SIE_BIT
}

#[inline]
fn local_irq_restore(flags: usize) {
    // restore the `SIE` bit
    unsafe { asm!("csrrs x0, sstatus, {}", in(reg) flags) };
}
