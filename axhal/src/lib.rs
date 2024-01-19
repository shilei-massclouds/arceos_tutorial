#![no_std]
#![feature(asm_const)]
#![feature(naked_functions)]

#[cfg(target_arch = "riscv64")]
mod riscv64;
#[cfg(target_arch = "riscv64")]
pub use self::riscv64::*;

#[cfg(not(target_arch = "riscv64"))]
pub mod dummy {
    pub struct TaskContext;
    impl TaskContext {
        pub const fn new() -> Self {
            Self
        }
        pub fn init(&mut self, _entry: usize, _kstack_top: usize) {
            unimplemented!();
        }
        pub fn switch_to(&mut self, _next_ctx: &Self) {
            unimplemented!();
        }
    }
}

#[cfg(not(target_arch = "riscv64"))]
pub use self::dummy::*;
