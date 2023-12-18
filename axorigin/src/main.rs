#![no_std]
#![no_main]
#![feature(asm_const)]

mod lang_items;
mod boot;

unsafe extern "C" fn rust_entry(_hartid: usize, _dtb: usize) {
    core::arch::asm!(
        "wfi",
        options(noreturn)
    )
}
