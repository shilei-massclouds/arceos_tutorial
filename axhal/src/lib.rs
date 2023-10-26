#![no_std]
#![feature(asm_const)]

mod lang_items;
mod boot;
mod paging;

pub mod console;

unsafe extern "C" fn rust_entry(hartid: usize, dtb: usize) {
    extern "C" {
        fn main(hartid: usize, dtb: usize);
    }

    main(hartid, dtb);
}
