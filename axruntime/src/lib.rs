#![no_std]

pub use axhal::ax_println as println;

#[macro_use]
extern crate axlog;

#[no_mangle]
pub extern "C" fn rust_main(hartid: usize, dtb: usize) -> ! {
    extern "C" {
        fn _skernel();
        fn main();
    }

    println!("\nArceOS is starting ...");

    // We reserve 2M memory range [0x80000000, 0x80200000) for SBI,
    // but it only occupies ~194K. Split this range in half,
    // requisition the higher part(1M) for early heap.
    axalloc::early_init(_skernel as usize - 0x100000, 0x100000);

    axlog::init();
    axlog::set_max_level(option_env!("LOG").unwrap_or("")); // no effect if set `log-level-*` features
    info!("Logging is enabled.");
    info!("Primary CPU {} started, dtb = {:#x}.", hartid, dtb);

    unsafe { main(); }
    panic!("ArceOS exit ...");
}
