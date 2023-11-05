use spinlock::SpinNoIrq;
use crate::{AxTaskRef, WaitQueue};
use crate::task::{CurrentTask, TaskState, Task};
use alloc::sync::Arc;
use alloc::collections::VecDeque;
use crate::task::current;
use axsync::BootOnceCell;

static EXITED_TASKS: SpinNoIrq<VecDeque<AxTaskRef>> = SpinNoIrq::new(VecDeque::new());
static WAIT_FOR_EXIT: WaitQueue = WaitQueue::new();

pub(crate) static RUN_QUEUE: SpinNoIrq<AxRunQueue> = SpinNoIrq::new(AxRunQueue::new());
static IDLE_TASK: BootOnceCell<AxTaskRef> = unsafe { BootOnceCell::new() };

pub(crate) struct AxRunQueue {
    ready_queue: VecDeque<Arc<Task>>,
}

impl AxRunQueue {
    pub const fn new() -> Self {
        Self { ready_queue: VecDeque::new() }
    }

    pub fn scheduler_timer_tick(&mut self) {
        let curr = current();
        if !curr.is_idle() && curr.task_tick() {
            curr.set_preempt_pending(true);
        }
    }

    pub fn preempt_resched(&mut self) {
        let curr = current();
        assert!(curr.is_running());

        // When we get the mutable reference of the run queue, we must
        // have held the `SpinNoIrq` lock with both IRQs and preemption
        // disabled. So we need to set `current_disable_count` to 1 in
        // `can_preempt()` to obtain the preemption permission before
        //  locking the run queue.
        let can_preempt = curr.can_preempt(1);

        debug!(
            "current task is to be preempted: {}, allow={}",
            curr.name(),
            can_preempt
        );
        if can_preempt {
            self.resched(true);
        } else {
            curr.set_preempt_pending(true);
        }
    }

    pub fn add_task(&mut self, task: AxTaskRef) {
        debug!("task spawn: {}", task.name());
        //assert!(task.is_ready());
        self.ready_queue.push_back(task);
    }

    pub fn pick_next_task(&mut self) -> Option<Arc<Task>> {
        self.ready_queue.pop_front()
    }

    pub fn put_prev_task(&mut self, prev: Arc<Task>, preempt: bool) {
        if prev.time_slice() > 0 && preempt {
            self.ready_queue.push_front(prev)
        } else {
            prev.reset_time_slice();
            self.ready_queue.push_back(prev)
        }
    }

    pub fn yield_current(&mut self) {
        self.resched(false);
    }
}

impl AxRunQueue {
    fn resched(&mut self, preempt: bool) {
        let prev = current();
        if prev.is_running() {
            prev.set_state(TaskState::Ready);
            if !prev.is_idle() {
                self.put_prev_task(prev.clone(), preempt);
            }
        }
        let next = self.pick_next_task().unwrap_or_else(|| IDLE_TASK.get().clone());
        self.switch_to(prev, next);
    }

    fn switch_to(&mut self, prev_task: CurrentTask, next_task: AxTaskRef) {
        next_task.set_preempt_pending(false);
        next_task.set_state(TaskState::Running);
        if prev_task.ptr_eq(&next_task) {
            return;
        }

        unsafe {
            let prev_ctx_ptr = prev_task.ctx_mut_ptr();
            let next_ctx_ptr = next_task.ctx_mut_ptr();

            assert!(Arc::strong_count(prev_task.as_task_ref()) > 1);
            assert!(Arc::strong_count(&next_task) >= 1);

            CurrentTask::set_current(prev_task, next_task);
            (*prev_ctx_ptr).switch_to(&*next_ctx_ptr);
        }
    }

    pub fn exit_current(&mut self, exit_code: i32) -> ! {
        let curr = current();
        debug!("task exit: {}, exit_code={}", curr.name(), exit_code);
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

    pub fn unblock_task(&mut self, task: AxTaskRef, resched: bool) {
        debug!("task unblock: {}", task.name());
        if task.is_blocked() {
            task.set_state(TaskState::Ready);
            self.add_task(task); // TODO: priority
            if resched {
                current().set_preempt_pending(true);
            }
        }
    }

    pub fn block_current<F>(&mut self, wait_queue_push: F)
    where
        F: FnOnce(AxTaskRef),
    {
        let curr = current();
        debug!("task block: {}", curr.name());
        assert!(curr.is_running());
        assert!(!curr.is_idle());
        assert!(curr.can_preempt(1));
        curr.set_state(TaskState::Blocked);
        wait_queue_push(curr.clone());
        self.resched(false);
    }
}

fn gc_entry() {
    loop {
        let n = EXITED_TASKS.lock().len();
        for _ in 0..n {
            let task = EXITED_TASKS.lock().pop_front();
            if let Some(task) = task {
                if Arc::strong_count(&task) == 1 {
                    drop(task);
                } else {
                    EXITED_TASKS.lock().push_back(task);
                }
            }
        }
        WAIT_FOR_EXIT.wait();
    }
}

pub(crate) fn init() {
    const IDLE_TASK_STACK_SIZE: usize = 4096;
    let idle_task = Task::new(|| run_idle(), "idle".into(), IDLE_TASK_STACK_SIZE);
    IDLE_TASK.init(idle_task.clone());

    let gc_task = Task::new(gc_entry, "gc".into(), axconfig::TASK_STACK_SIZE);
    RUN_QUEUE.lock().add_task(gc_task);

    let main_task = Task::new_init("main".into());
    main_task.set_state(TaskState::Running);

    unsafe { CurrentTask::init_current(main_task) }
}

pub fn yield_now() {
    RUN_QUEUE.lock().yield_current();
}

pub fn run_idle() -> ! {
    loop {
        yield_now();
    }
}
