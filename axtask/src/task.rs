use core::{alloc::Layout, cell::UnsafeCell, ptr::NonNull};
use alloc::{boxed::Box, sync::Arc};
use core::sync::atomic::{AtomicUsize, Ordering};
use axconfig::{PAGE_SIZE, align_up};
use axhal::TaskContext;

pub type AxTaskRef = Arc<Task>;

pub struct Task {
    entry: Option<*mut dyn FnOnce()>,
    kstack: Option<TaskStack>,
    ctx: UnsafeCell<TaskContext>,
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

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
    fn new_common() -> Self {
        Self { entry: None, kstack: None, ctx: UnsafeCell::new(TaskContext::new()) }
    }

    pub(crate) fn new<F>(entry: F) -> AxTaskRef
    where
        F: FnOnce() + 'static,
    {
        let mut t = Self::new_common();
        let kstack = TaskStack::alloc(align_up(PAGE_SIZE, PAGE_SIZE));

        t.entry = Some(Box::into_raw(Box::new(entry)));
        t.ctx.get_mut().init(task_entry as usize, kstack.top());
        t.kstack = Some(kstack);
        Arc::new(t)
    }

    pub(crate) fn new_init() -> AxTaskRef {
        Arc::new(Self::new_common())
    }

    #[inline]
    pub(crate) const unsafe fn ctx_mut_ptr(&self) -> *mut TaskContext {
        self.ctx.get()
    }
}

static NEXT_TASK: AtomicUsize = AtomicUsize::new(0);
static MAIN_TASK: AtomicUsize = AtomicUsize::new(0);

extern "C" fn task_entry() -> ! {
    let ptr: *const Task = NEXT_TASK.load(Ordering::Relaxed) as *const Task;
    let task = unsafe { AxTaskRef::from_raw(ptr) };
    if let Some(entry) = task.entry {
        unsafe { Box::from_raw(entry)() };
    }
    loop {}
}

pub(crate) fn init() {
    let busy_task = Task::new(|| run_busy());
    let next_ptr = Arc::into_raw(busy_task.clone());
    NEXT_TASK.store(next_ptr as usize, Ordering::Relaxed);

    let main_task = Task::new_init();
    let main_ptr = Arc::into_raw(main_task.clone());
    MAIN_TASK.store(main_ptr as usize, Ordering::Relaxed);

    info!("Try switch task ...");
    switch_to(main_task, busy_task);
    info!("Switch task ok!");
}

fn switch_to(prev_task: AxTaskRef, next_task: AxTaskRef) {
    unsafe {
        let prev_ctx_ptr = prev_task.ctx_mut_ptr();
        let next_ctx_ptr = next_task.ctx_mut_ptr();
        (*prev_ctx_ptr).switch_to(&*next_ctx_ptr);
    }
}

pub fn run_busy() -> ! {
    info!("I'm busy!");
    let busy_ptr: *const Task = NEXT_TASK.load(Ordering::Relaxed) as *const Task;
    let busy_task = unsafe { AxTaskRef::from_raw(busy_ptr) };
    let main_ptr: *const Task = MAIN_TASK.load(Ordering::Relaxed) as *const Task;
    let main_task = unsafe { AxTaskRef::from_raw(main_ptr) };
    switch_to(busy_task, main_task);
    loop {}
}
