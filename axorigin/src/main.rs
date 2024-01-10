#![no_std]
#![no_main]

use axstd::{String, println, time, PAGE_SIZE};

#[no_mangle]
pub fn main(_hartid: usize, _dtb: usize) {
    let now = time::Instant::now();
    println!("\nNow: {}", now);

    let s = String::from("from String");
    println!("Hello, ArceOS![{}]", s);

    try_alloc_pages();

    let d = now.elapsed();
    println!("Elapsed: {}.{:06}", d.as_secs(), d.subsec_micros());
}

fn try_alloc_pages() {
    use core::alloc::Layout;
    extern crate alloc;

    const NUM_PAGES:usize = 300;
    let layout = Layout::from_size_align(NUM_PAGES*PAGE_SIZE, PAGE_SIZE).unwrap();
    let p = unsafe { alloc::alloc::alloc(layout) };
    println!("Allocate pages: [{:?}].", p);
    unsafe { alloc::alloc::dealloc(p, layout) };
    println!("Release pages ok!");
}
