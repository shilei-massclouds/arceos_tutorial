#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

mod task;

pub fn init_sched() {
    task::init();
}
