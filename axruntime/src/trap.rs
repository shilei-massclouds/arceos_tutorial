#[cfg(all(target_os = "none", not(test)))]
struct TrapHandlerImpl;

#[cfg(all(target_os = "none", not(test)))]
#[crate_interface::impl_interface]
impl axhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_irq(irq_num: usize) {
        axhal::irq::dispatch_irq(irq_num);
    }
}
