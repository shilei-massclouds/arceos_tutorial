//! os:v2: introduce axhal.

#![no_std]
#![no_main]
#![feature(asm_const)]

mod boot;
mod mem;
mod paging;

pub mod console;

unsafe extern "C" fn rust_entry(hartid: usize, dtb: usize) {
    extern "C" {
        fn rust_main(hartid: usize, dtb: usize);
    }

    mem::clear_bss();

    rust_main(hartid, dtb);
}
