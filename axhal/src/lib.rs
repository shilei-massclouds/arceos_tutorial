#![no_std]
#![feature(asm_const)]

mod lang_items;
mod boot;
mod mem;
pub mod console;

unsafe extern "C" fn rust_entry(hartid: usize, dtb: usize) {
    extern "C" {
        fn main(hartid: usize, dtb: usize);
    }

    mem::clear_bss();

    main(hartid, dtb);
}
