#![no_std]
#![no_main]

#[no_mangle]
pub fn main(_hartid: usize, _dtb: usize) {
    axhal::console::write_bytes(b"\nHello, ArceOS!\n");
}
