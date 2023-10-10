//! app:v4: build major registry.

#![no_std]
#![no_main]

extern crate alloc;

use axstd::print;

const PAGE_SIZE: usize = 4096;

#[no_mangle]
pub fn main() {
    print!("\nHello, ArceOS! Final init allocator!\n");
    unsafe {
        let layout = alloc::alloc::Layout::from_size_align(PAGE_SIZE, PAGE_SIZE);
        let ptr = alloc::alloc::alloc(layout.unwrap());
        let page = core::slice::from_raw_parts_mut(ptr, PAGE_SIZE);
        page[PAGE_SIZE - 1] = 255;
        print!("Alloc one page ok! {:?}\n", page.get(PAGE_SIZE-1));
    }
    let mut v = alloc::vec::Vec::new();
    for i in 0..1000 {
        v.push(i);
    }
    print!("Alloc vec and expand allocator ok! {}\n", v.len());
}
