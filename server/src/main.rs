//! app:v4: build major registry.

#![no_std]
#![no_main]

use axstd::print;

#[no_mangle]
pub fn main() {
    print!("\nHello, ArceOS! Parse dtb!\n");
}
