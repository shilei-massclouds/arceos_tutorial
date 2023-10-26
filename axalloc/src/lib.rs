#![no_std]

mod early;

use core::alloc::{GlobalAlloc, Layout};
use crate::early::EarlyAllocator;
use axsync::BootCell;

struct GlobalAllocator {
    early_alloc: BootCell<EarlyAllocator>,
}

impl GlobalAllocator {
    pub const fn new() -> Self {
        Self {
            early_alloc: unsafe {
                BootCell::new(EarlyAllocator::new())
            },
        }
    }

    pub fn early_init(&self, start: usize, len: usize) {
        self.early_alloc.access().init(start, len)
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.early_alloc.access().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.early_alloc.access().dealloc(ptr, layout)
    }
}

#[cfg_attr(all(target_os = "none", not(test)), global_allocator)]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

pub fn early_init(start: usize, len: usize) {
    GLOBAL_ALLOCATOR.early_init(start, len)
}
