use riscv::register::sstatus;

/// Shutdown the whole system, including all CPUs.
pub fn terminate() -> ! {
    axlog::info!("Shutting down...");
    sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::NoReason);
    axlog::warn!("It should shutdown!");
    loop {
        halt();
    }
}

/// Halt the current CPU.
#[inline]
pub fn halt() {
    disable_irqs();
    unsafe { riscv::asm::wfi() } // should never return
}

/// Makes the current CPU to ignore interrupts.
#[inline]
pub fn disable_irqs() {
    unsafe { sstatus::clear_sie() }
}
