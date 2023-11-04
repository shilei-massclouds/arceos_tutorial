pub mod console {
    /// Write a slice of bytes to the console.
    pub fn write_bytes(_bytes: &[u8]) {
        unimplemented!()
    }
}

pub mod time {
    use core::time::Duration;
    pub type TimeValue = Duration;
    pub fn current_time() -> TimeValue {
        unimplemented!()
    }
}

pub mod misc {
    /// Shutdown the whole system, including all CPUs.
    pub fn terminate() -> ! {
        unimplemented!()
    }
}

pub mod mem {
    pub const fn phys_to_virt(_pa: usize) -> usize {
        0
    }
}

pub mod context {
    pub struct TaskContext;

    impl TaskContext {
        pub const fn new() -> Self {
            Self
        }

        pub fn init(&mut self, _entry: usize, _kstack_top: usize) {
            unimplemented!();
        }

        pub fn switch_to(&mut self, _next_ctx: &Self) {
            unimplemented!();
        }
    }
}
pub use context::{TaskContext};

pub mod cpu {
    pub fn current_task_ptr<T>() -> *const T {
        unimplemented!()
    }
    pub unsafe fn set_current_task_ptr<T>(_ptr: *const T) {}
}
