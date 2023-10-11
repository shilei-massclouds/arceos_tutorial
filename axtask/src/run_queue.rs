use spinlock::SpinNoIrq;
use lazy_init::LazyInit;
use crate::{AxTaskRef, Scheduler, WaitQueue};
use scheduler::BaseScheduler;
use crate::task::{CurrentTask, TaskState, TaskInner};
use alloc::sync::Arc;
use alloc::collections::VecDeque;

static EXITED_TASKS: SpinNoIrq<VecDeque<AxTaskRef>> = SpinNoIrq::new(VecDeque::new());

static WAIT_FOR_EXIT: WaitQueue = WaitQueue::new();

pub(crate) static RUN_QUEUE: LazyInit<SpinNoIrq<AxRunQueue>> = LazyInit::new();

static IDLE_TASK: LazyInit<AxTaskRef> = LazyInit::new();

pub(crate) struct AxRunQueue {
    scheduler: Scheduler,
}

impl AxRunQueue {
    pub fn new() -> SpinNoIrq<Self> {
        let gc_task = TaskInner::new(gc_entry, "gc".into(), axconfig::TASK_STACK_SIZE);
        let mut scheduler = Scheduler::new();
        scheduler.add_task(gc_task);
        SpinNoIrq::new(Self { scheduler })
    }

    pub fn add_task(&mut self, task: AxTaskRef) {
        debug!("task spawn: {}", task.id_name());
        assert!(task.is_ready());
        self.scheduler.add_task(task);
    }

    pub fn yield_current(&mut self) {
        let curr = crate::current();
        trace!("task yield: {}", curr.id_name());
        assert!(curr.is_running());
        self.resched(false);
    }

    /*
    pub fn set_current_priority(&mut self, prio: isize) -> bool {
        self.scheduler
            .set_priority(crate::current().as_task_ref(), prio)
    }
    */

    pub fn exit_current(&mut self, exit_code: i32) -> ! {
        let curr = crate::current();
        debug!("task exit: {}, exit_code={}", curr.id_name(), exit_code);
        assert!(curr.is_running());
        assert!(!curr.is_idle());
        if curr.is_init() {
            EXITED_TASKS.lock().clear();
            axhal::misc::terminate();
        } else {
            curr.set_state(TaskState::Exited);
            curr.notify_exit(exit_code, self);
            EXITED_TASKS.lock().push_back(curr.clone());
            WAIT_FOR_EXIT.notify_one_locked(false, self);
            self.resched(false);
        }
        unreachable!("task exited!");
    }

    pub fn block_current<F>(&mut self, wait_queue_push: F)
    where
        F: FnOnce(AxTaskRef),
    {
        let curr = crate::current();
        debug!("task block: {}", curr.id_name());
        assert!(curr.is_running());
        assert!(!curr.is_idle());

        curr.set_state(TaskState::Blocked);
        wait_queue_push(curr.clone());
        self.resched(false);
    }

    pub fn unblock_task(&mut self, task: AxTaskRef, resched: bool) {
        debug!("task unblock: {}", task.id_name());
        if task.is_blocked() {
            task.set_state(TaskState::Ready);
            self.scheduler.add_task(task); // TODO: priority
        }
    }
}

impl AxRunQueue {
    /// Common reschedule subroutine. If `preempt`, keep current task's time
    /// slice, otherwise reset it.
    fn resched(&mut self, preempt: bool) {
        let prev = crate::current();
        if prev.is_running() {
            prev.set_state(TaskState::Ready);
            if !prev.is_idle() {
                self.scheduler.put_prev_task(prev.clone(), preempt);
            }
        }
        let next = self.scheduler.pick_next_task().unwrap_or_else(|| unsafe {
            // Safety: IRQs must be disabled at this time.
            IDLE_TASK.get_unchecked().clone()
        });
        self.switch_to(prev, next);
    }

    fn switch_to(&mut self, prev_task: CurrentTask, next_task: AxTaskRef) {
        trace!(
            "context switch: {} -> {}",
            prev_task.id_name(),
            next_task.id_name()
        );
        next_task.set_state(TaskState::Running);
        if prev_task.ptr_eq(&next_task) {
            return;
        }

        unsafe {
            let prev_ctx_ptr = prev_task.ctx_mut_ptr();
            let next_ctx_ptr = next_task.ctx_mut_ptr();

            // The strong reference count of `prev_task` will be decremented by 1,
            // but won't be dropped until `gc_entry()` is called.
            assert!(Arc::strong_count(prev_task.as_task_ref()) > 1);
            assert!(Arc::strong_count(&next_task) >= 1);

            CurrentTask::set_current(prev_task, next_task);
            (*prev_ctx_ptr).switch_to(&*next_ctx_ptr);
        }
    }
}

fn gc_entry() {
    loop {
        // Drop all exited tasks and recycle resources.
        let n = EXITED_TASKS.lock().len();
        for _ in 0..n {
            // Do not do the slow drops in the critical section.
            let task = EXITED_TASKS.lock().pop_front();
            if let Some(task) = task {
                if Arc::strong_count(&task) == 1 {
                    // If I'm the last holder of the task, drop it immediately.
                    drop(task);
                } else {
                    // Otherwise (e.g, `switch_to` is not compeleted, held by the
                    // joiner, etc), push it back and wait for them to drop first.
                    EXITED_TASKS.lock().push_back(task);
                }
            }
        }
        WAIT_FOR_EXIT.wait();
    }
}

pub(crate) fn init() {
    const IDLE_TASK_STACK_SIZE: usize = 4096;
    let idle_task = TaskInner::new(|| crate::run_idle(), "idle".into(), IDLE_TASK_STACK_SIZE);
    IDLE_TASK.init_by(idle_task.clone());

    let main_task = TaskInner::new_init("main".into());
    main_task.set_state(TaskState::Running);

    RUN_QUEUE.init_by(AxRunQueue::new());
    unsafe { CurrentTask::init_current(main_task) }
}

/*
pub(crate) fn init_secondary() {
    let idle_task = TaskInner::new_init("idle".into());
    idle_task.set_state(TaskState::Running);
    IDLE_TASK.with_current(|i| i.init_by(idle_task.clone()));
    unsafe { CurrentTask::init_current(idle_task) }
}
*/
