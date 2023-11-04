static mut CURRENT_TASK_PTR: usize = 0;

#[inline]
pub fn current_task_ptr<T>() -> *const T {
    unsafe { CURRENT_TASK_PTR as _ }
}

#[inline]
pub unsafe fn set_current_task_ptr<T>(ptr: *const T) {
    CURRENT_TASK_PTR = ptr as usize
}
