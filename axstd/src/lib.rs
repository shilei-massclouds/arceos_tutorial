#![no_std]

extern crate alloc;
extern crate axruntime;

#[macro_use]
mod macros;

pub mod io;
pub mod thread;

// Re-export String
pub use alloc::string::String;

pub type Result<T> = axerrno::AxResult<T>;
