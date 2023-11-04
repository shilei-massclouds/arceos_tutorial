use core::ops::Deref;
use core::mem::ManuallyDrop;
use core::{alloc::Layout, cell::UnsafeCell, ptr::NonNull};
use alloc::{boxed::Box, string::String, sync::Arc};
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicI32, AtomicU64, Ordering};
use axconfig::{PAGE_SIZE, align_up};
use axhal::TaskContext;
use crate::WaitQueue;
use crate::run_queue::AxRunQueue;

pub type AxTaskRef = Arc<Task>;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

pub struct Task {
    id: TaskId,
    name: String,
    is_idle: bool,
    is_init: bool,
    entry: Option<*mut dyn FnOnce()>,
    state: AtomicU8,
    in_wait_queue: AtomicBool,
    exit_code: AtomicI32,
    wait_for_exit: WaitQueue,
    kstack: Option<TaskStack>,
    ctx: UnsafeCell<TaskContext>,
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

pub struct CurrentTask(ManuallyDrop<AxTaskRef>);

pub fn current() -> CurrentTask {
    CurrentTask::get()
}

/// The possible states of a task.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TaskState {
    Running = 1,
    Ready = 2,
    Blocked = 3,
    Exited = 4,
}

impl From<u8> for TaskState {
    #[inline]
    fn from(state: u8) -> Self {
        match state {
            1 => Self::Running,
            2 => Self::Ready,
            3 => Self::Blocked,
            4 => Self::Exited,
            _ => unreachable!(),
        }
    }
}

struct TaskStack {
    ptr: NonNull<u8>,
    layout: Layout,
}

impl TaskStack {
    pub fn alloc(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 16).unwrap();
        Self {
            ptr: NonNull::new(unsafe { alloc::alloc::alloc(layout) }).unwrap(),
            layout,
        }
    }

    pub const fn top(&self) -> usize {
        unsafe { core::mem::transmute(self.ptr.as_ptr().add(self.layout.size())) }
    }
}

impl Drop for TaskStack {
    fn drop(&mut self) {
        unsafe { alloc::alloc::dealloc(self.ptr.as_ptr(), self.layout) }
    }
}

impl Task {
    pub const fn id(&self) -> TaskId {
        self.id
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn join(&self) -> Option<i32> {
        self.wait_for_exit.wait_until(|| self.state() == TaskState::Exited);
        Some(self.exit_code.load(Ordering::Acquire))
    }
}

impl Task {
    fn new_common(id: TaskId, name: String) -> Self {
        Self {
            id,
            name,
            is_idle: false,
            is_init: false,
            entry: None,
            state: AtomicU8::new(TaskState::Ready as u8),
            in_wait_queue: AtomicBool::new(false),
            exit_code: AtomicI32::new(0),
            wait_for_exit: WaitQueue::new(),
            kstack: None,
            ctx: UnsafeCell::new(TaskContext::new()),
        }
    }

    /// Create a new task with the given entry function and stack size.
    pub(crate) fn new<F>(entry: F, name: String, stack_size: usize) -> AxTaskRef
    where
        F: FnOnce() + 'static,
    {
        let mut t = Self::new_common(TaskId::new(), name);
        debug!("new task: {}", t.name());
        let kstack = TaskStack::alloc(align_up(stack_size, PAGE_SIZE));

        t.entry = Some(Box::into_raw(Box::new(entry)));
        t.ctx.get_mut().init(task_entry as usize, kstack.top());
        t.kstack = Some(kstack);
        if t.name == "idle" {
            t.is_idle = true;
        }
        Arc::new(t)
    }

    pub(crate) fn new_init(name: String) -> AxTaskRef {
        let mut t = Self::new_common(TaskId::new(), name);
        t.is_init = true;
        if t.name == "idle" {
            t.is_idle = true;
        }
        Arc::new(t)
    }

    #[inline]
    pub(crate) fn state(&self) -> TaskState {
        self.state.load(Ordering::Acquire).into()
    }

    #[inline]
    pub(crate) fn set_state(&self, state: TaskState) {
        self.state.store(state as u8, Ordering::Release)
    }

    #[inline]
    pub(crate) fn is_running(&self) -> bool {
        matches!(self.state(), TaskState::Running)
    }

    #[inline]
    pub(crate) const fn is_idle(&self) -> bool {
        self.is_idle
    }
    #[inline]
    pub(crate) const fn is_init(&self) -> bool {
        self.is_init
    }
    #[inline]
    pub(crate) fn is_blocked(&self) -> bool {
        matches!(self.state(), TaskState::Blocked)
    }
    #[inline]
    pub(crate) fn in_wait_queue(&self) -> bool {
        self.in_wait_queue.load(Ordering::Acquire)
    }
    #[inline]
    pub(crate) fn set_in_wait_queue(&self, in_wait_queue: bool) {
        self.in_wait_queue.store(in_wait_queue, Ordering::Release);
    }

    pub(crate) fn notify_exit(&self, exit_code: i32, rq: &mut AxRunQueue) {
        self.exit_code.store(exit_code, Ordering::Release);
        self.wait_for_exit.notify_all_locked(false, rq);
    }

    #[inline]
    pub(crate) const unsafe fn ctx_mut_ptr(&self) -> *mut TaskContext {
        self.ctx.get()
    }
}

impl CurrentTask {
    pub(crate) fn try_get() -> Option<Self> {
        let ptr: *const Task = axhal::cpu::current_task_ptr();
        if !ptr.is_null() {
            Some(Self(unsafe { ManuallyDrop::new(AxTaskRef::from_raw(ptr)) }))
        } else {
            None
        }
    }

    pub(crate) fn get() -> Self {
        Self::try_get().expect("current task is uninitialized")
    }

    pub(crate) fn clone(&self) -> AxTaskRef {
        self.0.deref().clone()
    }

    pub(crate) fn ptr_eq(&self, other: &AxTaskRef) -> bool {
        Arc::ptr_eq(&self.0, other)
    }

    pub(crate) unsafe fn init_current(init_task: AxTaskRef) {
        let ptr = Arc::into_raw(init_task);
        axhal::cpu::set_current_task_ptr(ptr);
    }

    pub(crate) unsafe fn set_current(prev: Self, next: AxTaskRef) {
        let Self(arc) = prev;
        ManuallyDrop::into_inner(arc); // `call Arc::drop()` to decrease prev task reference count.
        let ptr = Arc::into_raw(next);
        axhal::cpu::set_current_task_ptr(ptr);
    }
}

impl Deref for CurrentTask {
    type Target = Task;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

extern "C" fn task_entry() -> ! {
    let task = current();
    if let Some(entry) = task.entry {
        unsafe { Box::from_raw(entry)() };
    }
    crate::exit(0);
}
