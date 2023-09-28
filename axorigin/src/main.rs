#![no_std]
#![no_main]
#![feature(asm_const)]

mod lang_items;
mod boot;
mod console;
mod mem;

unsafe extern "C" fn rust_entry(_hartid: usize, _dtb: usize) {
    mem::clear_bss();
    console::write_bytes(b"\nHello, ArceOS!\n");
}
