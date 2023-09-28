#![no_std]
#![no_main]

mod lang_items;

#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn _start() -> ! {
    core::arch::asm!(
        "wfi",
        options(noreturn)
    )
}
