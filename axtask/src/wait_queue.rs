use alloc::collections::VecDeque;
use spinlock::SpinRaw;
use crate::{AxTaskRef, run_queue::RUN_QUEUE};
use crate::task::{current, CurrentTask};
use crate::run_queue::AxRunQueue;

pub struct WaitQueue {
    queue: SpinRaw<VecDeque<AxTaskRef>>, // we already disabled IRQs when lock the `RUN_QUEUE`
}

impl WaitQueue {
    pub const fn new() -> Self {
        Self {
            queue: SpinRaw::new(VecDeque::new()),
        }
    }

    pub fn wait_until<F>(&self, condition: F)
    where
        F: Fn() -> bool,
    {
        loop {
            let mut rq = RUN_QUEUE.lock();
            if condition() {
                break;
            }
            rq.block_current(|task| {
                task.set_in_wait_queue(true);
                self.queue.lock().push_back(task);
            });
        }
        self.cancel_events(current());
    }

    fn cancel_events(&self, curr: CurrentTask) {
        if curr.in_wait_queue() {
            let _guard = kernel_guard::IrqSave::new();
            self.queue.lock().retain(|t| !curr.ptr_eq(t));
            curr.set_in_wait_queue(false);
        }
    }

    pub(crate) fn notify_all_locked(&self, resched: bool, rq: &mut AxRunQueue) {
        while let Some(task) = self.queue.lock().pop_front() {
            task.set_in_wait_queue(false);
            rq.unblock_task(task, resched);
        }
    }
    pub(crate) fn notify_one_locked(&self, resched: bool, rq: &mut AxRunQueue) -> bool {
        if let Some(task) = self.queue.lock().pop_front() {
            task.set_in_wait_queue(false);
            rq.unblock_task(task, resched);
            true
        } else {
            false
        }
    }

    pub fn wait(&self) {
        RUN_QUEUE.lock().block_current(|task| {
            task.set_in_wait_queue(true);
            self.queue.lock().push_back(task)
        });
        self.cancel_events(current());
    }

    pub fn notify_one(&self, resched: bool) -> bool {
        let mut rq = RUN_QUEUE.lock();
        if !self.queue.lock().is_empty() {
            self.notify_one_locked(resched, &mut rq)
        } else {
            false
        }
    }

    pub fn notify_all(&self, resched: bool) {
        loop {
            let mut rq = RUN_QUEUE.lock();
            if let Some(task) = self.queue.lock().pop_front() {
                task.set_in_wait_queue(false);
                rq.unblock_task(task, resched);
            } else {
                break;
            }
            drop(rq); // we must unlock `RUN_QUEUE` after unlocking `self.queue`.
        }
    }
}
