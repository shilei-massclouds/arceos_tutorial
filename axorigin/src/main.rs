#![no_std]
#![no_main]

extern crate alloc;
extern crate axruntime;

use alloc::string::String;

#[no_mangle]
pub fn main() {
    let s = String::from("\nHello, ArceOS![from String]\n");
    axhal::console::write_bytes(s.as_bytes());
}
