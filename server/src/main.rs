//! app:v4: build major registry.

#![no_std]
#![no_main]

use axstd::{print, println, String};

#[no_mangle]
pub fn main() {
    let info = String::from("\nHello, ArceOS! Allocate it!\n");
    print!("{}", info);
}
