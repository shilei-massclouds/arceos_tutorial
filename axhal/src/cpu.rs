use core::sync::atomic::{AtomicUsize, Ordering};

/// Sets the pointer to the current task with preemption-safety.
///
/// Preemption may be enabled when calling this function. This function will
/// guarantee the correctness even the current task is preempted.
static CURRENT_TASK_PTR: AtomicUsize = AtomicUsize::new(0);

///
/// # Safety
///
/// The given `ptr` must be pointed to a valid task structure.
#[inline]
pub unsafe fn set_current_task_ptr<T>(ptr: *const T) {
    let _guard = kernel_guard::IrqSave::new();
    CURRENT_TASK_PTR.store(ptr as usize, Ordering::Relaxed);
}

/// Gets the pointer to the current task with preemption-safety.
///
/// Preemption may be enabled when calling this function. This function will
/// guarantee the correctness even the current task is preempted.
#[inline]
pub fn current_task_ptr<T>() -> *const T {
    unsafe {
        // on RISC-V, reading `CURRENT_TASK_PTR` requires multiple instruction, so we disable local IRQs.
        let _guard = kernel_guard::IrqSave::new();
        CURRENT_TASK_PTR.load(Ordering::Relaxed) as _
    }
}
