#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::String;
use axhal::ax_println;

#[no_mangle]
pub fn main(_hartid: usize, _dtb: usize) {
    extern "C" {
        fn _skernel();
    }
    // We reserve 2M memory range [0x80000000, 0x80200000) for SBI,
    // but it only occupies ~194K. Split this range in half,
    // requisition the higher part(1M) for early heap.
    axalloc::early_init(_skernel as usize - 0x100000, 0x100000);

    let s = String::from("from String");
    ax_println!("\nHello, ArceOS![{}]", s);
}
