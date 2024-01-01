#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::String;

pub fn main(_hartid: usize, _dtb: usize) {
    let s = String::from("\nHello, ArceOS![from String]\n");
    axhal::console::write_bytes(s.as_bytes());
}
