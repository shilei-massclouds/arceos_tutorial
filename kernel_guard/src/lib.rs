#![no_std]
#![feature(asm_const)]

#[crate_interface::def_interface]
pub trait KernelGuardIf {
    fn enable_preempt();
    fn disable_preempt();
}

pub trait BaseGuard {
    type State: Clone + Copy;
    fn acquire() -> Self::State;
    fn release(state: Self::State);
}

pub struct NoOp;
pub struct IrqSave(usize);
pub struct NoPreempt;
pub struct NoPreemptIrqSave(usize);

impl BaseGuard for NoOp {
    type State = ();
    fn acquire() -> Self::State {}
    fn release(_state: Self::State) {}
}

impl NoOp {
    pub const fn new() -> Self { Self }
}

impl Drop for NoOp {
    fn drop(&mut self) {}
}

impl BaseGuard for IrqSave {
    type State = usize;

    #[inline]
    fn acquire() -> Self::State {
        arch::local_irq_save_and_disable()
    }

    #[inline]
    fn release(state: Self::State) {
        arch::local_irq_restore(state);
    }
}

impl BaseGuard for NoPreempt {
    type State = ();
    fn acquire() -> Self::State {
        crate_interface::call_interface!(KernelGuardIf::disable_preempt);
    }
    fn release(_state: Self::State) {
        crate_interface::call_interface!(KernelGuardIf::enable_preempt);
    }
}

impl BaseGuard for NoPreemptIrqSave {
    type State = usize;
    fn acquire() -> Self::State {
        crate_interface::call_interface!(KernelGuardIf::disable_preempt);
        arch::local_irq_save_and_disable()
    }
    fn release(state: Self::State) {
        arch::local_irq_restore(state);
        crate_interface::call_interface!(KernelGuardIf::enable_preempt);
    }
}

impl IrqSave {
    pub fn new() -> Self {
        Self(Self::acquire())
    }
}

impl Drop for IrqSave {
    fn drop(&mut self) {
        Self::release(self.0)
    }
}

impl Default for IrqSave {
    fn default() -> Self {
        Self::new()
    }
}

impl NoPreempt {
    pub fn new() -> Self {
        Self::acquire();
        Self
    }
}

impl Drop for NoPreempt {
    fn drop(&mut self) {
        Self::release(())
    }
}

impl Default for NoPreempt {
    fn default() -> Self {
        Self::new()
    }
}

impl NoPreemptIrqSave {
    pub fn new() -> Self {
        Self(Self::acquire())
    }
}

impl Drop for NoPreemptIrqSave {
    fn drop(&mut self) {
        Self::release(self.0)
    }
}

impl Default for NoPreemptIrqSave {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "riscv64")]
mod arch {
    use core::arch::asm;

    /// Bit 1: Supervisor Interrupt Enable
    const SIE_BIT: usize = 1 << 1;

    #[inline]
    pub fn local_irq_save_and_disable() -> usize {
        let flags: usize;
        unsafe { asm!("csrrc {}, sstatus, {}", out(reg) flags, const SIE_BIT) };
        flags & SIE_BIT
    }

    #[inline]
    pub fn local_irq_restore(flags: usize) {
        unsafe { asm!("csrrs x0, sstatus, {}", in(reg) flags) };
    }
}

#[cfg(not(target_arch = "riscv64"))]
mod arch {
    pub fn local_irq_save_and_disable() -> usize {
        unimplemented!()
    }
    pub fn local_irq_restore(_flags: usize) {
        unimplemented!();
    }
}
