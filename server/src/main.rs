//! app:v4: build major registry.

#![no_std]
#![no_main]

extern crate alloc;

use axstd::println;
use axstd::thread;

const PAGE_SIZE: usize = 4096;

#[no_mangle]
pub fn main() {
    println!("Hello, ArceOS! Start task...");

    let computation = thread::spawn(|| {
        42
    });

    let result = computation.join().unwrap();
    println!("Task gets result: {result}");
}
