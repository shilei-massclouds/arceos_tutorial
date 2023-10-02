#![no_std]
#![no_main]

use axstd::{println, String};

#[no_mangle]
pub fn main() {
    let s = String::from("Hello, ArceOS!");
    println!("\n{s} Now axstd is okay!");
}
