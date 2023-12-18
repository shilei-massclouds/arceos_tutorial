#![no_std]
#![no_main]

mod lang_items;
mod boot;
pub mod console;

unsafe extern "C" fn rust_entry(_hartid: usize, _dtb: usize) {
    extern "C" {
        fn main(hartid: usize, dtb: usize);
    }
    main(_hartid, _dtb);
}
