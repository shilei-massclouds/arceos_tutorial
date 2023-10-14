mod mutex;

use core::time::Duration;
pub use self::mutex::{Mutex, MutexGuard};

/// A handle to a wait queue.
///
/// A wait queue is used to store sleeping tasks waiting for a certain event
/// to happen.
pub struct AxWaitQueueHandle(axtask::WaitQueue);

impl AxWaitQueueHandle {
    /// Creates a new empty wait queue.
    pub const fn new() -> Self {
        Self(axtask::WaitQueue::new())
    }
}

pub fn ax_current_task_id() -> u64 {
    axtask::current().id().as_u64()
}

pub fn ax_wait_queue_wait(
    wq: &AxWaitQueueHandle,
    until_condition: impl Fn() -> bool,
    timeout: Option<Duration>,
) -> bool {
    if let Some(dur) = timeout {
        return wq.0.wait_timeout_until(dur, until_condition);
    }

    if timeout.is_some() {
        panic!("ax_wait_queue_wait: the `timeout` argument is ignored without the `irq` feature");
    }
    wq.0.wait_until(until_condition);
    false
}

pub fn ax_wait_queue_wake(wq: &AxWaitQueueHandle, count: u32) {
    if count == u32::MAX {
        wq.0.notify_all(true);
    } else {
        for _ in 0..count {
            wq.0.notify_one(true);
        }
    }
}
