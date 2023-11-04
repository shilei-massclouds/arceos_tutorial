use spinlock::SpinRaw;
use crate::{AxTaskRef, WaitQueue};
use crate::task::{CurrentTask, TaskState, Task};
use alloc::sync::Arc;
use alloc::collections::VecDeque;
use crate::task::current;
use axconfig::PAGE_SIZE;
use axsync::BootOnceCell;

static EXITED_TASKS: SpinRaw<VecDeque<AxTaskRef>> = SpinRaw::new(VecDeque::new());
static WAIT_FOR_EXIT: WaitQueue = WaitQueue::new();

pub(crate) static RUN_QUEUE: SpinRaw<AxRunQueue> = SpinRaw::new(AxRunQueue::new());
static IDLE_TASK: BootOnceCell<AxTaskRef> = unsafe { BootOnceCell::new() };

pub(crate) struct AxRunQueue {
    ready_queue: VecDeque<Arc<Task>>,
}

impl AxRunQueue {
    pub const fn new() -> Self {
        Self { ready_queue: VecDeque::new() }
    }

    pub fn add_task(&mut self, task: AxTaskRef) {
        debug!("task spawn: {}", task.name());
        //assert!(task.is_ready());
        self.ready_queue.push_back(task);
    }

    pub fn pick_next_task(&mut self) -> Option<Arc<Task>> {
        self.ready_queue.pop_front()
    }

    pub fn put_prev_task(&mut self, prev: Arc<Task>, _preempt: bool) {
        self.ready_queue.push_back(prev);
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
        let next = self.pick_next_task().unwrap();
        self.switch_to(prev, next);
    }

    fn switch_to(&mut self, prev_task: CurrentTask, next_task: AxTaskRef) {
        next_task.set_state(TaskState::Running);
        if prev_task.ptr_eq(&next_task) {
            return;
        }

        unsafe {
            let prev_ctx_ptr = prev_task.ctx_mut_ptr();
            let next_ctx_ptr = next_task.ctx_mut_ptr();

            CurrentTask::set_current(prev_task, next_task);
            (*prev_ctx_ptr).switch_to(&*next_ctx_ptr);
        }
    }
    pub fn block_current<F>(&mut self, wait_queue_push: F)
    where
        F: FnOnce(AxTaskRef),
    {
        let curr = current();
        assert!(curr.is_running());
        assert!(!curr.is_idle());

        curr.set_state(TaskState::Blocked);
        wait_queue_push(curr.clone());
        self.resched(false);
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
        }
    }
}

pub(crate) fn init() {
    const IDLE_TASK_STACK_SIZE: usize = 4096;
    let idle_task = Task::new(|| run_idle(), "idle".into(), IDLE_TASK_STACK_SIZE);
    IDLE_TASK.init(idle_task.clone());

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
