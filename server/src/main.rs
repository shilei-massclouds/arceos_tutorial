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
    let mut s = axstd::String::from("ABC");
    s.push('D');
    print!("Alloc string ok! {}\n", s);
}
