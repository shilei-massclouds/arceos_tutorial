#![no_std]
#![feature(asm_const)]

mod boot;
mod paging;

pub mod console;

unsafe extern "C" fn rust_entry(hartid: usize, dtb: usize) {
    extern "C" {
        fn rust_main(hartid: usize, dtb: usize);
    }

    rust_main(hartid, dtb);
}
