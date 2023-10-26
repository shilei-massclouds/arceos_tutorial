#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

#[no_mangle]
pub fn main(_hartid: usize, _dtb: usize) {
    // We reserve 2M memory range [0x80000000, 0x80200000) for SBI,
    // but it only occupies ~194K. Split this range in half,
    // requisition the higher part(1M) for early heap.
    axalloc::early_init(_skernel as usize - 0x100000, 0x100000);

    let s = String::from("\nHello, ArceOS![from String]\n");
    axhal::console::write_bytes(s.as_bytes());
}

extern "C" {
    fn _skernel();
}
