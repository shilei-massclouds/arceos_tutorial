#![no_std]

use core::alloc::Layout;
use core::ptr::NonNull;
use spinlock::SpinRaw;
use axconfig::PAGE_SIZE;

extern crate alloc;
use alloc::alloc::GlobalAlloc;

mod early;
use early::EarlyAllocator;

#[derive(Debug)]
pub enum AllocError {
    InvalidParam,
    MemoryOverlap,
    NoMemory,
    NotAllocated,
}

pub type AllocResult<T = ()> = Result<T, AllocError>;

#[cfg_attr(not(test), global_allocator)]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

struct GlobalAllocator {
    early_alloc: SpinRaw<EarlyAllocator>,
}

impl GlobalAllocator {
    pub const fn new() -> Self {
        Self {
            early_alloc: SpinRaw::new(EarlyAllocator::uninit_new()),
        }
    }

    pub fn early_init(&self, start: usize, size: usize) {
        self.early_alloc.lock().init(start, size)
    }

    fn alloc_bytes(&self, layout: Layout) -> *mut u8 {
        if let Ok(ptr) = self.early_alloc.lock().alloc_bytes(layout) {
            ptr.as_ptr()
        } else {
            alloc::alloc::handle_alloc_error(layout)
        }
    }
    fn dealloc_bytes(&self, ptr: *mut u8, layout: Layout) {
        self.early_alloc.lock().dealloc_bytes(
            NonNull::new(ptr).expect("dealloc null ptr"),
            layout
        )
    }
	fn alloc_pages(&self, layout: Layout) -> *mut u8 {
        if let Ok(ptr) = self.early_alloc.lock().alloc_pages(layout) {
            ptr.as_ptr()
        } else {
            alloc::alloc::handle_alloc_error(layout)
        }
    }
    fn dealloc_pages(&self, _ptr: *mut u8, _layout: Layout) {
        unimplemented!();
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() % PAGE_SIZE == 0 && layout.align() == PAGE_SIZE {
            self.alloc_pages(layout)
        } else {
            self.alloc_bytes(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() % PAGE_SIZE == 0 && layout.align() == PAGE_SIZE {
            self.dealloc_pages(ptr, layout)
        } else {
            self.dealloc_bytes(ptr, layout)
        }
    }
}

pub fn early_init(start: usize, len: usize) {
    GLOBAL_ALLOCATOR.early_init(start, len)
}
