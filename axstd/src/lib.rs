#![no_std]

extern crate alloc;
pub use alloc::string::String;
pub use axruntime::println;

pub mod time;
pub use time::*;
