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

/// A guard that disables/enables local IRQs around the critical section.
pub struct IrqSave(usize);

impl BaseGuard for IrqSave {
    type State = usize;

    #[inline]
    fn acquire() -> Self::State {
        local_irq_save_and_disable()
    }

    #[inline]
    fn release(state: Self::State) {
        // restore IRQ states
        local_irq_restore(state);
    }
}

impl IrqSave {
    /// Creates a new [`IrqSave`] guard.
    pub fn new() -> Self {
        Self(Self::acquire())
    }
}

impl Drop for IrqSave {
    fn drop(&mut self) {
        Self::release(self.0)
    }
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

/// A no-op guard that does nothing around the critical section.
pub struct NoOp;

impl BaseGuard for NoOp {
    type State = ();
    fn acquire() -> Self::State {}
    fn release(_state: Self::State) {}
}

impl NoOp {
    /// Creates a new [`NoOp`] guard.
    pub const fn new() -> Self {
        Self
    }
}

impl Drop for NoOp {
    fn drop(&mut self) {}
}
