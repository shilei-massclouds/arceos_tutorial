#![no_std]
#![no_main]

use axstd::{String, println, time};

#[no_mangle]
pub fn main(_hartid: usize, _dtb: usize) {
    let now = time::Instant::now();
    println!("\nNow: {}", now);

    let s = String::from("from String");
    println!("Hello, ArceOS![{}]", s);

    let d = now.elapsed();
    println!("Elapsed: {}.{:06}", d.as_secs(), d.subsec_micros());
}
