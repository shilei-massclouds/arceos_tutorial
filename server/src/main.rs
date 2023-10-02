//! app:v4: build major registry.

#![no_std]
#![no_main]

use axstd::{print, println, String};

#[no_mangle]
pub fn main(_hartid: usize, dtb: usize) {
    let info = String::from("\nHello, ArceOS! Allocate it!\n");
    print!("{}", info);
    println!("dtb: {:x}, kernel: {:x}", dtb, _skernel as usize);
}

extern "C" {
    fn _skernel();
}
