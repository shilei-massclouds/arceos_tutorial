#![no_std]

extern crate alloc;
extern crate axruntime;

#[macro_use]
mod macros;

pub mod io;

// Re-export String
pub use alloc::string::String;
