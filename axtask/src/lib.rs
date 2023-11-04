#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

mod task;
mod run_queue;

pub use task::AxTaskRef;

pub fn init_sched() {
    run_queue::init();
}
