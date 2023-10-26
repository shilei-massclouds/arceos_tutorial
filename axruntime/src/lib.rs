#![no_std]

#[cfg(all(target_os = "none", not(test)))]
mod lang_items;

#[macro_use]
extern crate axlog;

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
    extern "C" {
        fn _skernel();
        #[cfg(not(test))]
        fn main();
    }

    let log_level = option_env!("AX_LOG").unwrap_or("");
    ax_println!("\nArceOS is starting... [{}]\n", log_level);

    axlog::init();
    axlog::set_max_level(log_level);
    info!("Logging is enabled.");
    info!("Primary CPU {} started, dtb = {:#x}.", hartid, dtb);

    // We reserve 2M memory range [0x80000000, 0x80200000) for SBI,
    // but it only occupies ~194K. Split this range in half,
    // requisition the higher part(1M) for early heap.
    axalloc::early_init(_skernel as usize - 0x100000, 0x100000);

    #[cfg(not(test))]
    unsafe {
        main();
    }

    debug!("main task exited: exit_code={}", 0);
    axhal::misc::terminate();
}
