//! os:v2: introduce axhal.

#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(const_maybe_uninit_zeroed)]
#![feature(const_option)]
#![feature(doc_auto_cfg)]

#[macro_use]
extern crate axlog;

#[macro_use]
mod macros;

mod boot;

pub mod trap;
pub mod irq;
pub mod context;
pub mod paging;
pub mod cpu;
pub mod mem;
pub mod misc;
pub mod time;
pub mod console;

unsafe extern "C" fn rust_entry(hartid: usize, dtb: usize) {
    extern "C" {
        fn trap_vector_base();
        fn rust_main(hartid: usize, dtb: usize);
    }

    mem::clear_bss();
    trap::set_trap_vector_base(trap_vector_base as usize);

    rust_main(hartid, dtb);
}

/// Initializes the platform devices for the primary CPU.
///
/// For example, the interrupt controller and the timer.
pub fn platform_init() {
    self::irq::init_percpu();
    self::time::init_percpu();
}
