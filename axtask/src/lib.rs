#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::string::String;

mod task;
mod run_queue;
mod wait_queue;

pub use task::AxTaskRef;
pub use wait_queue::WaitQueue;

pub fn init_sched() {
    run_queue::init();
}

pub fn spawn_raw<F>(f: F, name: String, stack_size: usize) -> AxTaskRef
where
    F: FnOnce() + 'static,
{
    let task = task::Task::new(f, name, stack_size);
    run_queue::RUN_QUEUE.lock().add_task(task.clone());
    task
}

pub fn init_scheduler() {
    info!("Initialize scheduling...");
    run_queue::init();
}

pub fn exit(exit_code: i32) -> ! {
    run_queue::RUN_QUEUE.lock().exit_current(exit_code)
}
