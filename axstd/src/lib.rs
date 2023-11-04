#![no_std]

extern crate alloc;
extern crate axruntime;

#[macro_use]
mod macros;

pub mod io;
pub mod time;
pub mod thread;

// Re-export String
pub use alloc::string::String;
pub use alloc::vec::Vec;
