#![no_std]

use core::alloc::Layout;
use core::ptr::NonNull;
use spinlock::SpinRaw;
use axconfig::PAGE_SIZE;
use axsync::BootOnceCell;

extern crate alloc;
use alloc::alloc::GlobalAlloc;

mod early;
use early::EarlyAllocator;

mod bitmap;
use bitmap::BitmapPageAllocator;

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
    page_alloc: SpinRaw<BitmapPageAllocator>,
    finalized: BootOnceCell<bool>,
}

impl GlobalAllocator {
    pub const fn new() -> Self {
        Self {
            early_alloc: SpinRaw::new(EarlyAllocator::uninit_new()),
            page_alloc: SpinRaw::new(BitmapPageAllocator::new()),
            finalized: BootOnceCell::new(),
        }
    }

    pub fn early_init(&self, start: usize, size: usize) {
        self.early_alloc.lock().init(start, size)
    }
    pub fn final_init(&self, start: usize, size: usize) {
        self.page_alloc.lock().init(start, size);
        self.finalized.init(true);
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
        let ret = if self.finalized.is_init() {
            self.page_alloc.lock().alloc_pages(layout)
        } else {
            self.early_alloc.lock().alloc_pages(layout)
        };

        if let Ok(ptr) = ret {
            ptr.as_ptr()
        } else {
            alloc::alloc::handle_alloc_error(layout)
        }
    }
    fn dealloc_pages(&self, ptr: *mut u8, layout: Layout) {
        if self.finalized.is_init() {
            self.page_alloc.lock().dealloc_pages(ptr as usize, layout.size()/PAGE_SIZE)
        } else {
            unimplemented!()
        };
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
pub fn final_init(start: usize, len: usize) {
    GLOBAL_ALLOCATOR.final_init(start, len)
}
