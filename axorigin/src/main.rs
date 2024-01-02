#![no_std]
#![no_main]

use axstd::{String, println};

#[no_mangle]
pub fn main(_hartid: usize, _dtb: usize) {
    let s = String::from("from String");
    println!("\nHello, ArceOS![{}]", s);
}
