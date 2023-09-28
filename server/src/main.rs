//! v0: first line in kernel.

#![no_std]
#![no_main]
#![feature(asm_const)]

mod app;

mod lang_items;
mod boot;
mod mem;
mod console;

unsafe extern "C" fn rust_entry(hartid: usize, dtb: usize) {
    mem::clear_bss();

    app::main(hartid, dtb);
}
