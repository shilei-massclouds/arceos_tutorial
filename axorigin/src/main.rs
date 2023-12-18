#![no_std]
#![no_main]

use axhal::ax_println;

#[no_mangle]
pub fn main(_hartid: usize, _dtb: usize) {
    let version = 1;
    ax_println!("\nHello, ArceOS!");
    ax_println!("version: [{}]", version);
}
