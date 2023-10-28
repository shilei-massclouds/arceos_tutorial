#![no_std]

use core::ptr::NonNull;
use core::alloc::{GlobalAlloc, Layout};
use allocator::{BaseAllocator, ByteAllocator, EarlyAllocator};
use axsync::BootCell;

extern crate alloc;

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

    pub fn available_bytes(&self) -> usize {
        self.early_alloc.access().available_bytes()
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Ok(ptr) = self.early_alloc.access().alloc(layout) {
            ptr.as_ptr()
        } else {
            alloc::alloc::handle_alloc_error(layout)
        }

    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.early_alloc.access().dealloc(
            NonNull::new(ptr).expect("dealloc null ptr"),
            layout
        )
    }
}

#[cfg_attr(all(target_os = "none", not(test)), global_allocator)]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

pub fn early_init(start: usize, len: usize) {
    GLOBAL_ALLOCATOR.early_init(start, len)
}

pub fn available_bytes() -> usize {
    GLOBAL_ALLOCATOR.available_bytes()
}
