use axhal::TaskContext;

static mut CUR_CTX: TaskContext = TaskContext::new();
static mut NEXT_CTX: TaskContext = TaskContext::new();

pub fn init_scheduler() {
    info!("Try switch task ...");
    let layout = core::alloc::Layout::from_size_align(4096, 16).unwrap();
    let kstack = unsafe { alloc::alloc::alloc(layout) };
    let kstack_top = kstack as usize + 4096;

    unsafe {
        NEXT_CTX.init(task_entry as usize, kstack_top);
        CUR_CTX.switch_to(&NEXT_CTX);
    }
    info!("Switch task ok!");
}

extern "C" fn task_entry() -> ! {
    info!("Now in another task!");
    unsafe {
        NEXT_CTX.switch_to(&CUR_CTX);
    }
    loop {}
}
