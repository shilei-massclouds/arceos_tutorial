#![no_std]

#[macro_use]
extern crate axlog;

mod lang_items;

extern "C" {
    fn _skernel();
    fn main();
}

struct LogIfImpl;

#[crate_interface::impl_interface]
impl axlog::LogIf for LogIfImpl {
    fn console_write_str(s: &str) {
        axhal::console::write_bytes(s.as_bytes());
    }

    fn current_time() -> core::time::Duration {
        axhal::time::current_time()
    }
}

#[no_mangle]
pub extern "C" fn rust_main(hartid: usize, dtb: usize) -> ! {
    let log_level = option_env!("AX_LOG").unwrap_or("");
    ax_println!("\nArceOS is starting... [{}]\n", log_level);

    axlog::init();
    axlog::set_max_level(log_level);
    info!("Logging is enabled.");
    info!("Primary CPU {} started, dtb = {:#x}.", hartid, dtb);

    #[cfg(feature = "alloc")]
    {
        // For Riscv64 memory layout, we reserve 2M memory space for SBI.
        // Now SBI just occupies about 194K, so reserve 1M for it and
        // requisition another 1M for early heap.
        axalloc::global_init(_skernel as usize - 0x100000, 0x100000);
    }

    unsafe {
        main();
    }

    debug!("main task exited: exit_code={}", 0);
    axhal::misc::terminate();
}
