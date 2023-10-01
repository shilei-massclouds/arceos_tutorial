//! app:v4: build major registry.

#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

#[no_mangle]
pub fn main(_hartid: usize, _dtb: usize) {
    // For Riscv64 memory layout, we reserve 2M memory space for SBI.
    // Now SBI just occupies about 194K, so reserve 1M for it and
    // requisition another 1M for early heap.
    axalloc::global_init(_skernel as usize - 0x100000, 0x100000);

    let info = String::from("\nHello, ArceOS! Allocate it!\n");
    axhal::console::write_bytes(info.as_bytes());
}

extern "C" {
    fn _skernel();
}
