#![no_std]

mod lang_items;

#[no_mangle]
pub extern "C" fn rust_main(_hartid: usize, _dtb: usize) -> ! {
    extern "C" {
        fn _skernel();
        fn main();
    }

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

    panic!("Never reach here!");
}
