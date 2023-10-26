#![no_std]
#![no_main]

use axstd::{println, String, time};

#[no_mangle]
pub fn main() {
    let now = time::Instant::now();

    let s = String::from("Hello, ArceOS!");
    println!("{s} Now axstd is okay!");

    let d = now.elapsed();
    println!("Elapsed: {}.{:06}", d.as_secs(), d.subsec_micros());
}
