//! Global memory allocator

#![no_std]

use core::ptr::NonNull;
use core::alloc::{GlobalAlloc, Layout};
use allocator::{AllocResult, BaseAllocator, ByteAllocator};
use allocator::{EarlyAllocator, TlsfByteAllocator};

use axsync::BootCell;

extern crate alloc;

struct GlobalAllocator {
    early_alloc: BootCell<EarlyAllocator>,
    byte_alloc: BootCell<TlsfByteAllocator>
}

impl GlobalAllocator {
    pub const fn new() -> Self {
        Self {
            early_alloc: unsafe {
                BootCell::new(EarlyAllocator::new())
            },
            byte_alloc: unsafe {
                BootCell::new(TlsfByteAllocator::new())
            },
        }
    }

    pub fn early_init(&self, start: usize, size: usize) {
        self.early_alloc.exclusive_access().init(start, size)
    }

    pub fn final_init(&self, start: usize, size: usize) {
        self.byte_alloc.exclusive_access().init(start, size)
    }

    pub fn final_add_memory(&self, start: usize, size: usize) -> AllocResult {
        self.byte_alloc.exclusive_access().add_memory(start, size)
    }

    pub fn available_bytes(&self) -> usize {
        self.early_alloc.exclusive_access().available_bytes()
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.byte_alloc.exclusive_access().total_bytes() > 0 {
            if let Ok(ptr) = self.byte_alloc.exclusive_access().alloc(layout) {
                return ptr.as_ptr();
            } else {
                alloc::alloc::handle_alloc_error(layout)
            }
        }

        // Final allocator hasn't initialized yet, use early allocator.
        if let Ok(ptr) = self.early_alloc.exclusive_access().alloc(layout) {
            ptr.as_ptr()
        } else {
            alloc::alloc::handle_alloc_error(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.early_alloc.exclusive_access().dealloc(
            NonNull::new(ptr).expect("dealloc null ptr"),
            layout
        )
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

pub fn early_init(start: usize, size: usize) {
    GLOBAL_ALLOCATOR.early_init(start, size)
}

pub fn final_init(start: usize, size: usize) {
    GLOBAL_ALLOCATOR.final_init(start, size)
}

pub fn final_add_memory(start: usize, size: usize) -> AllocResult {
    GLOBAL_ALLOCATOR.final_add_memory(start, size)
}

pub fn available_bytes() -> usize {
    GLOBAL_ALLOCATOR.available_bytes()
}
