#![no_std]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(const_maybe_uninit_zeroed)]
#![feature(const_option)]
#![feature(doc_auto_cfg)]

#[cfg(target_arch = "riscv64")]
mod riscv64;
#[cfg(target_arch = "riscv64")]
pub use self::riscv64::*;

#[cfg(not(target_arch = "riscv64"))]
mod dummy;
#[cfg(not(target_arch = "riscv64"))]
pub use self::dummy::*;
