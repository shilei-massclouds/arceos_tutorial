#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

mod task;
pub use task::init_scheduler;
