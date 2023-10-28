#![no_std]

use core::ptr::NonNull;
use core::alloc::{GlobalAlloc, Layout};
use allocator::{BaseAllocator, ByteAllocator, PageAllocator};
use allocator::{EarlyAllocator, BitmapPageAllocator, AllocResult};
use axsync::BootCell;

extern crate alloc;

const PAGE_SIZE: usize = 4096;
const MIN_HEAP_SIZE: usize = 0x8000; // 32 K

struct GlobalAllocator {
    early_alloc: BootCell<EarlyAllocator>,
    page_alloc: BootCell<BitmapPageAllocator>
}

impl GlobalAllocator {
    pub const fn new() -> Self {
        Self {
            early_alloc: unsafe {
                BootCell::new(EarlyAllocator::new())
            },
            page_alloc: unsafe {
                BootCell::new(BitmapPageAllocator::new())
            },
        }
    }

    pub fn early_init(&self, start: usize, size: usize) {
        self.early_alloc.access().init(start, size)
    }

    pub fn final_init(&self, start: usize, size: usize) {
        assert!(size > MIN_HEAP_SIZE);
        //let layout = Layout::from_size_align(MIN_HEAP_SIZE, PAGE_SIZE).unwrap();
        self.page_alloc.access().init(start, size);
        self.early_alloc.access().disable();
        //let heap_ptr = self.alloc_pages(layout) as usize;
        //self.byte_alloc.access().init(heap_ptr, MIN_HEAP_SIZE);
    }

    /*
    pub fn final_add_memory(&self, start: usize, size: usize) -> AllocResult {
        self.byte_alloc.access().add_memory(start, size)
    }
    */

    pub fn total_bytes(&self) -> usize {
        self.early_alloc.access().total_bytes()
    }

    pub fn available_bytes(&self) -> usize {
        self.early_alloc.access().available_bytes()
    }

    pub fn used_bytes(&self) -> usize {
        let alloc = self.early_alloc.access();
        alloc.used_bytes() + (alloc.used_pages() * PAGE_SIZE)
    }

    pub fn used_pages(&self) -> usize {
        if self.page_alloc.access().total_pages() > 0 {
            self.page_alloc.access().used_pages()
        } else {
            self.early_alloc.access().used_pages()
        }
    }

    fn alloc_pages(&self, layout: Layout) -> *mut u8 {
        assert!(layout.align() % PAGE_SIZE == 0);
        assert!(layout.size() % PAGE_SIZE == 0);
        let num = layout.size() / PAGE_SIZE;
        if self.page_alloc.access().total_pages() > 0 {
            if let Ok(ptr) = self.page_alloc.access().alloc_pages(num, layout.align()) {
                return ptr as *mut u8;
            } else {
                alloc::alloc::handle_alloc_error(layout)
            }
        }
        self.early_alloc(layout)
    }

    fn alloc_bytes(&self, layout: Layout) -> *mut u8 {
        /*
        if self.byte_alloc.access().total_bytes() > 0 {
            if let Ok(ptr) = self.byte_alloc.access().alloc(layout) {
                return ptr.as_ptr();
            } else {
                alloc::alloc::handle_alloc_error(layout)
            }
        }
        */
        self.early_alloc(layout)
    }

    fn early_alloc(&self, layout: Layout) -> *mut u8 {
        // Final allocator hasn't initialized yet, use early allocator.
        if let Ok(ptr) = self.early_alloc.access().alloc(layout) {
            ptr.as_ptr()
        } else {
            alloc::alloc::handle_alloc_error(layout)
        }

    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() % PAGE_SIZE == 0 {
            assert_eq!(layout.align(), PAGE_SIZE);
            self.alloc_pages(layout)
        } else {
            self.alloc_bytes(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if self.early_alloc.access().disabled() {
            if layout.size() % PAGE_SIZE == 0 {
                let num = layout.size() >> axconfig::PAGE_SHIFT;
                self.page_alloc.access().dealloc_pages(
                    ptr as usize,
                    num
                )
            } else {
                self.early_alloc.access().dealloc(
                    NonNull::new(ptr).expect("dealloc null ptr"),
                    layout
                )
            }
        } else {
            self.early_alloc.access().dealloc(
                NonNull::new(ptr).expect("dealloc null ptr"),
                layout
            )
        }
    }
}

#[cfg_attr(all(target_os = "none", not(test)), global_allocator)]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

pub fn early_init(start: usize, len: usize) {
    GLOBAL_ALLOCATOR.early_init(start, len)
}

pub fn final_init(start: usize, len: usize) {
    GLOBAL_ALLOCATOR.final_init(start, len)
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
