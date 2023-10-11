use alloc::{string::String, sync::Arc};

pub use crate::task::{CurrentTask, TaskId, TaskInner};
pub(crate) use crate::run_queue::{AxRunQueue, RUN_QUEUE};

pub(crate) type AxTask = scheduler::FifoTask<TaskInner>;
pub(crate) type Scheduler = scheduler::FifoScheduler<TaskInner>;

pub use crate::wait_queue::WaitQueue;

/// The reference type of a task.
pub type AxTaskRef = Arc<AxTask>;

/// Spawns a new task with the given parameters.
///
/// Returns the task reference.
pub fn spawn_raw<F>(f: F, name: String, stack_size: usize) -> AxTaskRef
where
    F: FnOnce() + 'static,
    //F: FnOnce() + Send + 'static,
{
    let task = TaskInner::new(f, name, stack_size);
    RUN_QUEUE.lock().add_task(task.clone());
    task
}

/*
//! Task APIs for multi-task configuration.


#[doc(cfg(feature = "multitask"))]


/// Gets the current task, or returns [`None`] if the current task is not
/// initialized.
pub fn current_may_uninit() -> Option<CurrentTask> {
    CurrentTask::try_get()
}
*/

/// Gets the current task.
///
/// # Panics
///
/// Panics if the current task is not initialized.
pub fn current() -> CurrentTask {
    CurrentTask::get()
}

/// Initializes the task scheduler (for the primary CPU).
pub fn init_scheduler() {
    info!("Initialize scheduling...");

    crate::run_queue::init();

    info!("  use {} scheduler.", Scheduler::scheduler_name());
}

/*
/// Spawns a new task with the default parameters.
///
/// The default task name is an empty string. The default task stack size is
/// [`axconfig::TASK_STACK_SIZE`].
///
/// Returns the task reference.
pub fn spawn<F>(f: F) -> AxTaskRef
where
    F: FnOnce() + Send + 'static,
{
    spawn_raw(f, "".into(), axconfig::TASK_STACK_SIZE)
}

/// Set the priority for current task.
///
/// The range of the priority is dependent on the underlying scheduler. For
/// example, in the [CFS] scheduler, the priority is the nice value, ranging from
/// -20 to 19.
///
/// Returns `true` if the priority is set successfully.
///
/// [CFS]: https://en.wikipedia.org/wiki/Completely_Fair_Scheduler
pub fn set_priority(prio: isize) -> bool {
    RUN_QUEUE.lock().set_current_priority(prio)
}
*/

/// Current task gives up the CPU time voluntarily, and switches to another
/// ready task.
pub fn yield_now() {
    RUN_QUEUE.lock().yield_current();
}

/*
/// Current task is going to sleep for the given duration.
///
/// If the feature `irq` is not enabled, it uses busy-wait instead.
pub fn sleep(dur: core::time::Duration) {
    sleep_until(axhal::time::current_time() + dur);
}

/// Current task is going to sleep, it will be woken up at the given deadline.
///
/// If the feature `irq` is not enabled, it uses busy-wait instead.
pub fn sleep_until(deadline: axhal::time::TimeValue) {
    #[cfg(feature = "irq")]
    RUN_QUEUE.lock().sleep_until(deadline);
    #[cfg(not(feature = "irq"))]
    axhal::time::busy_wait_until(deadline);
}
*/

/// Exits the current task.
pub fn exit(exit_code: i32) -> ! {
    RUN_QUEUE.lock().exit_current(exit_code)
}

/// The idle task routine.
///
/// It runs an infinite loop that keeps calling [`yield_now()`].
pub fn run_idle() -> ! {
    loop {
        yield_now();
    }
}
