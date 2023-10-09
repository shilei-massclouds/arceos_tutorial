//! Earliest memory allocator

use core::alloc::Layout;
use core::ptr::NonNull;
use crate::{AllocResult, AllocError, BaseAllocator, ByteAllocator};

extern crate alloc;

const PAGE_SIZE: usize = 4096;

#[macro_export]
macro_rules! ROUNDUP {
    ($a: expr, $b: expr) => {((($a) + (($b)-1)) & !(($b)-1))}
}

#[macro_export]
macro_rules! ROUNDDOWN {
    ($a: expr, $b: expr) => {(($a) & !(($b)-1))}
}

#[macro_export]
macro_rules! ALIGN_UP {
    ($a: expr, $b: expr) => {ROUNDUP!($a, $b)}
}

#[macro_export]
macro_rules! ALIGN_DOWN {
    ($a: expr, $b: expr) => {ROUNDDOWN!($a, $b)}
}

pub struct EarlyAllocator {
    start:  usize,
    end:    usize,
    next:   usize,
    count:  usize,
}

impl EarlyAllocator {
    pub const fn new() -> Self {
        Self {
            start:  0,
            end:    0,
            next:   0,
            count:  0,
        }
    }

    fn alloc_bytes(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        let start = ALIGN_UP!(self.next, layout.align());
        let end = start + layout.size();
        if end > self.end {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            self.next = end;
            self.count += 1;
            NonNull::new(start as *mut u8).ok_or(AllocError::NoMemory)
        }
    }

    fn alloc_pages(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        assert_eq!(layout.size() % PAGE_SIZE, 0);
        let end = ALIGN_DOWN!(self.end - layout.size(), layout.align());
        if end <= self.next {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            self.end = end;
            NonNull::new(end as *mut u8).ok_or(AllocError::NoMemory)
        }
    }
}

impl BaseAllocator for EarlyAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.next = start;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        todo!();
    }
}

impl ByteAllocator for EarlyAllocator {
    fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        if layout.size() % PAGE_SIZE == 0 {
            assert_eq!(layout.align(), PAGE_SIZE);
            self.alloc_pages(layout)
        } else {
            self.alloc_bytes(layout)
        }
    }

    fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
        self.count -= 1;
        if self.count == 0 {
            self.next = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.next - self.start
    }

    fn available_bytes(&self) -> usize {
        self.end - self.next
    }
}
