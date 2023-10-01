//! Global memory allocator

#![no_std]

mod early;

use core::alloc::{GlobalAlloc, Layout};
use crate::early::EarlyByteAllocator;

use core::cell::{RefCell, RefMut};

// Temporary wrapper. Replace it with Mutex/SpinXXX later!
struct BootCell<T> {
    inner: RefCell<T>,
}

impl<T> BootCell<T> {
    pub const unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value)
        }
    }

    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

unsafe impl<T> Sync for BootCell<T> {}

struct GlobalAllocator {
    early_alloc: BootCell<EarlyByteAllocator>,
}

impl GlobalAllocator {
    pub const fn new() -> Self {
        Self {
            early_alloc: unsafe {
                BootCell::new(EarlyByteAllocator::new())
            },
        }
    }

    pub fn init(&self, start: usize, len: usize) {
        self.early_alloc.exclusive_access().init(start, len)
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.early_alloc.exclusive_access().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.early_alloc.exclusive_access().dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

pub fn global_init(start: usize, len: usize) {
    GLOBAL_ALLOCATOR.init(start, len)
}
