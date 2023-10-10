//! Global memory allocator

#![no_std]

use core::ptr::NonNull;
use core::alloc::{GlobalAlloc, Layout};
use allocator::{AllocResult};
use allocator::{BaseAllocator, ByteAllocator, PageAllocator};
use allocator::{EarlyAllocator, TlsfByteAllocator, BitmapPageAllocator};

use axsync::BootCell;

#[macro_use]
extern crate log;
extern crate alloc;

const PAGE_SIZE: usize = 4096;
const MIN_HEAP_SIZE: usize = 0x8000; // 32 K

struct GlobalAllocator {
    early_alloc: BootCell<EarlyAllocator>,
    byte_alloc: BootCell<TlsfByteAllocator>,
    page_alloc: BootCell<BitmapPageAllocator>
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
            page_alloc: unsafe {
                BootCell::new(BitmapPageAllocator::new())
            },
        }
    }

    pub fn early_init(&self, start: usize, size: usize) {
        self.early_alloc.exclusive_access().init(start, size)
    }

    pub fn final_init(&self, start: usize, size: usize) {
        assert!(size > MIN_HEAP_SIZE);
        let layout = Layout::from_size_align(MIN_HEAP_SIZE, PAGE_SIZE).unwrap();
        self.page_alloc.exclusive_access().init(start, size);
        let heap_ptr = self.alloc_pages(layout) as usize;
        self.byte_alloc.exclusive_access().init(heap_ptr, MIN_HEAP_SIZE);
    }

    pub fn final_add_memory(&self, start: usize, size: usize) -> AllocResult {
        self.byte_alloc.exclusive_access().add_memory(start, size)
    }

    pub fn total_bytes(&self) -> usize {
        self.early_alloc.exclusive_access().total_bytes()
    }

    pub fn available_bytes(&self) -> usize {
        self.early_alloc.exclusive_access().available_bytes()
    }

    pub fn used_bytes(&self) -> usize {
        let alloc = self.early_alloc.exclusive_access();
        alloc.used_bytes() + (alloc.used_pages() * PAGE_SIZE)
    }

    pub fn used_pages(&self) -> usize {
        self.early_alloc.exclusive_access().used_pages()
    }

    fn alloc_pages(&self, layout: Layout) -> *mut u8 {
        assert!(layout.align() % PAGE_SIZE == 0);
        assert!(layout.size() % PAGE_SIZE == 0);
        let num = layout.size() / PAGE_SIZE;
        if self.page_alloc.exclusive_access().total_pages() > 0 {
            if let Ok(ptr) = self.page_alloc.exclusive_access().alloc_pages(num, layout.align()) {
                return ptr as *mut u8;
            } else {
                alloc::alloc::handle_alloc_error(layout)
            }
        }
        self.early_alloc(layout)
    }

    fn alloc_bytes(&self, layout: Layout) -> *mut u8 {
        if self.byte_alloc.exclusive_access().total_bytes() == 0 {
            return self.early_alloc(layout);
        }

        loop {
            let mut balloc = self.byte_alloc.exclusive_access();
            if let Ok(ptr) = balloc.alloc(layout) {
                return ptr.as_ptr();
            } else {
                let old_size = balloc.total_bytes();
                let expand_size = old_size
                    .max(layout.size())
                    .next_power_of_two()
                    .max(PAGE_SIZE);
                let layout = Layout::from_size_align(expand_size, PAGE_SIZE).unwrap();
                let heap_ptr = self.alloc_pages(layout) as usize;
                info!(
                    "expand heap memory: [{:#x}, {:#x})",
                    heap_ptr,
                    heap_ptr + expand_size
                );
                let _ = balloc.add_memory(heap_ptr, expand_size);
            }
        }
    }

    fn early_alloc(&self, layout: Layout) -> *mut u8 {
        // Final allocator hasn't initialized yet, use early allocator.
        if let Ok(ptr) = self.early_alloc.exclusive_access().alloc(layout) {
            ptr.as_ptr()
        } else {
            alloc::alloc::handle_alloc_error(layout)
        }
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

pub fn total_bytes() -> usize {
    GLOBAL_ALLOCATOR.total_bytes()
}

pub fn available_bytes() -> usize {
    GLOBAL_ALLOCATOR.available_bytes()
}

pub fn used_bytes() -> usize {
    GLOBAL_ALLOCATOR.used_bytes()
}

pub fn used_pages() -> usize {
    GLOBAL_ALLOCATOR.used_pages()
}
