#![no_std]
#![no_main]
#![feature(asm_const)]

mod lang_items;
mod boot;
mod console;

unsafe extern "C" fn rust_entry(_hartid: usize, _dtb: usize) {
    let version = 1;
    ax_println!("\nHello, ArceOS!");
    ax_println!("version: [{}]", version);
}
