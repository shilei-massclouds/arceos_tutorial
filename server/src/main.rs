//! app:v4: build major registry.

#![no_std]
#![no_main]

extern crate alloc;
extern crate axruntime;

use alloc::string::String;

#[no_mangle]
pub fn main(_hartid: usize, _dtb: usize) {
    let info = String::from("\nHello, ArceOS! Allocate it!\n");
    axhal::console::write_bytes(info.as_bytes());
}
