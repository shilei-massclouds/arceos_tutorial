use spinlock::SpinRaw;
use crate::AxTaskRef;
use crate::task::{CurrentTask, TaskState, Task};
use alloc::sync::Arc;
use alloc::collections::VecDeque;
use crate::task::current;

pub(crate) static RUN_QUEUE: SpinRaw<AxRunQueue> = SpinRaw::new(AxRunQueue::new());

pub(crate) struct AxRunQueue {
    ready_queue: VecDeque<Arc<Task>>,
}

impl AxRunQueue {
    pub const fn new() -> Self {
        Self { ready_queue: VecDeque::new() }
    }

    pub fn add_task(&mut self, task: AxTaskRef) {
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
}

pub(crate) fn init() {
    let busy_task = Task::new(|| run_busy(), "NOT busy".into());
    RUN_QUEUE.lock().add_task(busy_task);

    let main_task = Task::new_init("main".into());
    main_task.set_state(TaskState::Running);
    unsafe { CurrentTask::init_current(main_task) }

    info!("Try switch task ...");
    RUN_QUEUE.lock().yield_current();
    info!("Switch task ok!");
}

pub fn run_busy() -> ! {
    loop {
        info!("I'm NOT idle!");
        RUN_QUEUE.lock().yield_current();
    }
}
