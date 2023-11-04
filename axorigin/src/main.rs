#![no_std]
#![no_main]

use axstd::{println, String, time, Vec};

#[no_mangle]
pub fn main() {
    let now = time::Instant::now();

    let s = String::from("Hello, ArceOS!");
    println!("{s} Now axstd is okay!");

    try_alloc_bulk();

    axtask::init_sched();

    let d = now.elapsed();
    println!("Elapsed: {}.{:06}", d.as_secs(), d.subsec_micros());
}

fn try_alloc_bulk() {
    println!("\nTry alloc bulk memory ...\n");
    let mut v = Vec::new();
    for i in 0..0x2000 {
        v.push(i);
    }
    println!("Alloc bulk memory ok!\n");
}
